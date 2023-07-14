use sqlx::postgres::{PgPool, PgPoolOptions};

struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(3)
            .connect(database_url.as_ref())
            .await?;
        sqlx::migrate!().run(&db).await?;
        Ok(Self { pool: db })
    }

    // Users
    pub async fn add_user(&self, user: &UserData) -> anyhow::Result<UserData> {
        let user = sqlx::query_as!(
            UserData,
            r#"INSERT INTO
        users(login, name, is_admin, is_worker, pay, percent, pwd_hash, pwd_salt, token)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
            user.login,
            user.name,
            user.is_admin,
            user.is_worker,
            user.pay,
            user.percent,
            user.pwd_hash,
            user.pwd_salt,
            user.token
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user(&self, user_search: &UserSearch) -> anyhow::Result<UserData> {
        let user = match user_search {
            UserSearch::Id(id) => {
                sqlx::query_as!(UserData, r#"SELECT * FROM users WHERE id = $1"#, id)
                    .fetch_one(&self.pool)
                    .await?
            }

            UserSearch::Login(login) => {
                sqlx::query_as!(UserData, r#"SELECT * FROM users WHERE login = $1"#, login)
                    .fetch_one(&self.pool)
                    .await?
            }

            UserSearch::Token(token) => {
                sqlx::query_as!(UserData, r#"SELECT * FROM users WHERE token = $1"#, token)
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(user)
    }

    pub async fn update_user(&self, user: &UserData) -> anyhow::Result<UserData> {
        let user = sqlx::query_as!(
            UserData,
            r#"UPDATE users
        SET login = $2, name = $3, is_admin = $4, is_worker = $5, pay = $6,
        percent = $7, pwd_hash = $8, pwd_salt = $9, token = $10
        WHERE id = $1 RETURNING *"#,
            user.id,
            user.login,
            user.name,
            user.is_admin,
            user.is_worker,
            user.pay,
            user.percent,
            user.pwd_hash,
            user.pwd_salt,
            user.token
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    // Schedule
    pub async fn get_schedule(&self, month: u8, year: u16) -> anyhow::Result<Vec<ScheduleData>> {
        let schedule = sqlx::query_as!(
            ScheduleData,
            r#"SELECT * FROM schedule WHERE month = $1 AND year = $2"#,
            month as i32,
            year as i32,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(schedule)
    }

    pub async fn set_schedule(&self, schedule: &ScheduleData, working: bool) -> anyhow::Result<()> {
        if working {
            sqlx::query!(
                r#"INSERT INTO schedule VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"#,
                schedule.day,
                schedule.month,
                schedule.year,
                schedule.user_id
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"DELETE FROM schedule
                WHERE day = $1 AND month = $2 AND year = $3 AND user_id = $4"#,
                schedule.day,
                schedule.month,
                schedule.year,
                schedule.user_id
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    // Revenue
    pub async fn get_revenue(&self, month: u8, year: u16) -> anyhow::Result<Vec<RevenueData>> {
        let schedule = sqlx::query_as!(
            RevenueData,
            r#"SELECT * FROM revenue WHERE month = $1 AND year = $2"#,
            month as i32,
            year as i32,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(schedule)
    }

    pub async fn set_revenue(&self, revenue: &RevenueData) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO revenue VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(day, month, year) DO UPDATE
            SET with_percent = $4, without_percent = $5"#,
            revenue.day,
            revenue.month,
            revenue.year,
            revenue.with_percent,
            revenue.without_percent,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Payouts
    pub async fn get_payouts(&self, month: u8, year: u16) -> anyhow::Result<Vec<PayoutData>> {
        let schedule = sqlx::query_as!(
            PayoutData,
            r#"SELECT * FROM payouts WHERE month = $1 AND year = $2"#,
            month as i32,
            year as i32,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(schedule)
    }

    pub async fn add_payout(&self, payout: &PayoutData) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO payouts VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(day, month, year, user_id) DO UPDATE
            SET amount = $5"#,
            payout.day,
            payout.month,
            payout.year,
            payout.user_id,
            payout.amount,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Salary
    pub async fn get_salaries(&self, month: u8, year: u16) -> anyhow::Result<Vec<SalaryData>> {
        let schedule = sqlx::query_as!(
            SalaryData,
            r#"
SELECT u.id AS "user_id!",
       (u.pay * COALESCE(s.working_days, 0) + COALESCE(SUM(s.with_percent * u.percent / 100), 0)) AS "amount_owed!",
       COALESCE(p.amount_paid, 0) AS "amount_paid!"
FROM users u
LEFT JOIN
  (SELECT s.user_id,
          COUNT(*) AS working_days,
          SUM(r.with_percent / n.num_users) AS with_percent
   FROM schedule s
   JOIN revenue r ON s.day = r.day
   AND s.month = r.month
   AND s.year = r.year
   JOIN
     (SELECT DAY,
             MONTH,
             YEAR,
             COUNT(DISTINCT user_id) AS num_users
      FROM schedule
      WHERE month = $1
        AND year = $2
      GROUP BY DAY,
               MONTH,
               YEAR) n ON s.day = n.day
   AND s.month = n.month
   AND s.year = n.year
   WHERE s.month = $1
     AND s.year = $2
   GROUP BY s.user_id) s ON u.id = s.user_id
LEFT JOIN
  (SELECT user_id,
          SUM(amount) AS amount_paid
   FROM payouts
   WHERE month = $1
     AND year = $2
   GROUP BY user_id) p ON u.id = p.user_id
WHERE u.is_worker
GROUP BY u.id,
         u.pay,
         s.working_days,
         p.amount_paid;"#,
            month as i32,
            year as i32,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(schedule)
    }
}

enum UserSearch {
    Id(i32),
    Login(String),
    Token(String),
}

struct UserData {
    id: i32,
    login: String,
    name: String,
    is_admin: bool,
    is_worker: bool,
    pay: f64,
    percent: f64,
    pwd_hash: String,
    pwd_salt: String,
    token: String,
}

struct ScheduleData {
    day: i32,
    month: i32,
    year: i32,
    user_id: i32,
}

struct RevenueData {
    day: i32,
    month: i32,
    year: i32,
    with_percent: f64,
    without_percent: f64,
}

struct PayoutData {
    day: i32,
    month: i32,
    year: i32,
    user_id: i32,
    amount: f64,
}

struct SalaryData {
    user_id: i32,
    amount_paid: f64,
    amount_owed: f64,
}
