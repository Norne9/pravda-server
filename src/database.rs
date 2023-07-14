use async_trait::async_trait;

pub enum UserSearch {
    Id(i32),
    Login(String),
    Token(String),
}

pub struct UserData {
    pub id: i32,
    pub login: String,
    pub name: String,
    pub is_admin: bool,
    pub is_worker: bool,
    pub pay: f64,
    pub percent: f64,
    pub pwd_hash: String,
    pub pwd_salt: String,
    pub token: String,
}

pub struct ScheduleData {
    pub day: i32,
    pub month: i32,
    pub year: i32,
    pub user_id: i32,
}

pub struct RevenueData {
    pub day: i32,
    pub month: i32,
    pub year: i32,
    pub with_percent: f64,
    pub without_percent: f64,
}

pub struct PayoutData {
    pub day: i32,
    pub month: i32,
    pub year: i32,
    pub user_id: i32,
    pub amount: f64,
}

pub struct SalaryData {
    pub user_id: i32,
    pub amount_paid: f64,
    pub amount_owed: f64,
}

#[async_trait]
pub trait Database {
    // Users
    async fn add_user(&self, user: &UserData) -> anyhow::Result<UserData>;
    async fn get_user(&self, user_search: &UserSearch) -> anyhow::Result<UserData>;
    async fn get_users(&self, ids: Option<&Vec<i32>>) -> anyhow::Result<Vec<UserData>>;
    async fn update_user(&self, user: &UserData) -> anyhow::Result<UserData>;

    // Schedule
    async fn get_schedule(&self, month: u8, year: u16) -> anyhow::Result<Vec<ScheduleData>>;
    async fn set_schedule(&self, schedule: &ScheduleData, working: bool) -> anyhow::Result<()>;

    // Revenue
    async fn get_revenue(&self, month: u8, year: u16) -> anyhow::Result<Vec<RevenueData>>;
    async fn set_revenue(&self, revenue: &RevenueData) -> anyhow::Result<()>;

    // Payouts
    async fn get_payouts(&self, month: u8, year: u16) -> anyhow::Result<Vec<PayoutData>>;
    async fn add_payout(&self, payout: &PayoutData) -> anyhow::Result<()>;

    // Salary
    async fn get_salaries(&self, month: u8, year: u16) -> anyhow::Result<Vec<SalaryData>>;
}
