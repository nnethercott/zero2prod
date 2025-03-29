use sqlx::{Executor, PgPool, PgTransaction};
use uuid::Uuid;

use crate::{
    domain::SubscriberEmail,
    email_client::{EmailClient},
};

//NOTE: we're using updates on table state to drive email send job to completion
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<(), anyhow::Error> {
    if let Some((transaction, issue_id, email)) = dequeue_task(pool).await? {
        match SubscriberEmail::parse(email.clone()) {
            Ok(email) => {
                // NOTE: we only perform second query if email valid !
                let issue = get_issue(pool, &email_client, issue_id, email.as_ref()).await?;

                if let Err(e) = email_client
                    .send_email(
                        &email,
                        &issue.title,
                        &issue.text_content,
                        &issue.html_content,
                    )
                    .await
                {
                    // tracing failed to send
                }
            }
            Err(e) => {
                // tracing invalid email
            }
        }

        delete_task(transaction, issue_id, &email).await?;
    }
    Ok(())
}

// NOTE: - we wanna create a new transaction PER dequeue_task
// - return Result<Option ... since a) query could fail, and b) may be no rows left
async fn dequeue_task(pool: &PgPool) -> Result<Option<(PgTransaction, Uuid, String)>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT issue_id, email
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(r) = r {
        Ok(Some((transaction, r.issue_id, r.email)))
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
    email_client: &EmailClient,
    issue_id: Uuid,
    email: &str,
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
