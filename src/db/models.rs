use diesel::prelude::*;

use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use super::newtypes::UserId;
use super::pool::PgConn;
use super::schema::users;

/// User details.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

impl User {
    /// Run query using Diesel to find user by uid and return it.
    pub async fn find_user_by_uid(
        // Do not directly reference the database connection
        // type in the function signature. Use the synonym in pool.
        conn: &mut PgConn,
        user_id: UserId,
    ) -> Result<Option<User>, diesel::result::Error> {
        use super::schema::users::dsl::*;

        let user = users
            .filter(id.eq(user_id))
            .first::<User>(conn)
            .await
            .optional()?;

        Ok(user)
    }
}

/// New user details.
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
}

impl NewUser {
    pub async fn insert_new_user(
        conn: &mut PgConn,
        nm: &str, // prevent collision with `name` column imported inside the function
    ) -> Result<User, diesel::result::Error> {
        use super::schema::users::dsl::*;
        let new_user = NewUser {
            name: nm.to_owned(),
        };

        let user = diesel::insert_into(users).values(&new_user).get_result(conn).await?;

        Ok(user)
    }
}
