use std::time::Duration;

use sqlx::{Executor, PgPool, PgTransaction};
use uuid::Uuid;

use crate::{configuration::Settings, domain::SubscriberEmail, email_client::{EmailClient}, get_connection_pool};

pub enum ExecutionOutcome{
    EmptyQueue, 
    TaskCompleted,
    TaskFailed
}

pub async fn run_worker_until_stopped(config: Settings){
    let pool = get_connection_pool(&config.database);
    let email_client = config.email_client.client().expect("failed to parse email");

    let _ = worker_loop(&pool, &email_client).await;
}

// should this be yielding stuff for listensers?
pub async fn worker_loop(
    pool: &PgPool,
    email_client: &EmailClient,
)->Result<(), anyhow::Error>{
    loop{
        match try_execute_task(pool, email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }, 
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            _ => {}
        };
    }
}

//NOTE: we're using updates on table state to drive email send job to completion
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    
    if task.is_none(){
        return Ok(ExecutionOutcome::EmptyQueue);
    }

    if let Some((mut transaction, issue_id, email, retries)) = task {
        match SubscriberEmail::parse(email.clone()) {
            Ok(email) => {
                // NOTE: we only perform second query if email valid !
                let issue = get_issue(pool).await?;

                if let Err(_) = email_client
                    .send_email(
                        &email,
                        &issue.title,
                        &issue.text_content,
                        &issue.html_content,
                    )
                    .await
                {
                    // increment retries and bail
                    increment_retries(&mut transaction, issue_id, email.as_ref(), retries).await?;
                    transaction.rollback().await?;
                    return Ok(ExecutionOutcome::TaskFailed);
                }
            }
            Err(_) => {
                // tracing invalid email -- we'd wanna delete the task
            }
        }

        delete_task(transaction, issue_id, &email).await?;
    }
    Ok(ExecutionOutcome::TaskCompleted)
}

// NOTE: - we wanna create a new transaction PER dequeue_task
// - return Result<Option ... since a) query could fail, and b) may be no rows left
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String, i32)>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT issue_id, email, retries
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.issue_id,
            r.email,
            r.retries.unwrap_or(0)
        )))
    } else {
        Ok(None)
    }
}

async fn delete_task(
    mut transaction: PgTransaction<'_>,
    issue_id: Uuid,
    email: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue 
        WHERE 
            issue_id = $1 AND 
            email = $2
    "#,
        issue_id,
        email
    );
    transaction.execute(query).await?;
    transaction.commit().await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

async fn get_issue(
    pool: &PgPool,
) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
            SELECT 
                title,
                text_content,
                html_content
            FROM newsletter_issues
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(issue)
}

async fn increment_retries(
    transaction: &mut PgTransaction<'_>,
    issue_id: Uuid,
    email: &str,
    n_retries: i32,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(r#"
        UPDATE issue_delivery_queue
        SET retries = $3 
        WHERE issue_id = $1 AND
        email = $2 
    "#, issue_id, email, n_retries);

    transaction.execute(query).await?;
    Ok(())
}
