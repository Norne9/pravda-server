use crate::database::*;
use crate::utils;
use pravda_protocol::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct PravdaHandler<T: Database> {
    database: T,
}

impl<T: Database> PravdaHandler<T> {
    pub fn new(database: T) -> Self {
        Self { database }
    }

    pub async fn process(&self, request: Request, token: Option<String>) -> Response {
        if let Request::User(UserRequest::Login { login, password }) = request {
            return self.login(login, password).await;
        }

        let user = match token {
            Some(token) => match self.database.get_user(&UserSearch::Token(token)).await {
                Ok(user) => match user {
                    None => return Err(ProtocolError::UnknownToken),
                    Some(user) => user,
                },
                Err(e) => return Err(ProtocolError::Unknown(e.to_string())),
            },
            None => return Err(ProtocolError::Forbidden),
        };

        return match request {
            Request::User(user_request) => match user_request {
                UserRequest::Login { .. } => panic!("Can't be login here!"),
                UserRequest::GetUserInfo => Ok(ResponseData::UserInfo(User {
                    id: user.id,
                    login: user.login,
                    name: user.name,
                    is_admin: user.is_admin,
                    is_worker: user.is_worker,
                    pay: user.pay,
                    percent: user.percent,
                })),
                UserRequest::GetSchedule { year, month } => self.get_schedule(year, month).await,
                UserRequest::SetWorkday {
                    year,
                    month,
                    day,
                    is_working,
                } => {
                    self.set_workday(user.id, year, month, day, is_working)
                        .await
                }
                UserRequest::ChangePassword {
                    old_password,
                    new_password,
                } => self.set_password(user, old_password, new_password).await,
                UserRequest::GetUserNames { ids } => self.get_user_names(ids).await,
            },
            Request::Admin(admin_request) => {
                if !user.is_admin {
                    return Err(ProtocolError::Forbidden);
                }
                match admin_request {
                    AdminRequest::GetUsers => self.get_users().await,
                    AdminRequest::AddUser(user) => self.add_user(user).await,
                    AdminRequest::ResetPassword { id } => self.reset_password(id).await,
                    AdminRequest::UpdateUser(user) => self.update_user(user).await,
                    AdminRequest::GetRevenue { year, month } => self.get_revenue(year, month).await,
                    AdminRequest::SetRevenue {
                        year,
                        month,
                        revenue,
                    } => self.set_revenue(year, month, revenue).await,
                    AdminRequest::GetSalaryCalculation { year, month } => {
                        self.get_salary_calculation(year, month).await
                    }
                }
            }
        };
    }

    async fn login(&self, login: String, password: String) -> Response {
        let mut user = match self.database.get_user(&UserSearch::Login(login)).await {
            Ok(user) => match user {
                None => return Err(ProtocolError::LoginFailed),
                Some(user) => user,
            },
            Err(_) => return Err(ProtocolError::LoginFailed),
        };
        if user.pwd_hash == user.get_pwd_hash(password) {
            user.token = utils::make_uuid();
            match self.database.update_user(&user).await {
                Ok(user) => Ok(ResponseData::Login {
                    token: user.token,
                    id: user.id,
                }),
                Err(e) => Err(ProtocolError::Unknown(e.to_string())),
            }
        } else {
            Err(ProtocolError::LoginFailed)
        }
    }

    async fn get_schedule(&self, year: u16, month: u8) -> Response {
        let schedule = match self.database.get_schedule(month, year).await {
            Ok(s) => s,
            Err(e) => return Err(ProtocolError::Unknown(e.to_string())),
        };
        let days_in_month = utils::get_days_in_month(year, month);
        let false_vec = (0..=days_in_month)
            .into_iter()
            .map(|_| false)
            .collect::<Vec<bool>>();

        let users = schedule.iter().map(|s| s.user_id).collect::<HashSet<i32>>();
        let schedule = users
            .into_iter()
            .map(|uid| {
                let mut vec = false_vec.clone();
                for day in schedule
                    .iter()
                    .filter(|s| s.user_id == uid)
                    .map(|s| s.day as usize)
                {
                    vec[day] = true
                }
                (uid, vec)
            })
            .collect::<HashMap<i32, Vec<bool>>>();

        Ok(ResponseData::Schedule {
            year,
            month,
            schedule,
        })
    }

    async fn set_workday(
        &self,
        user_id: UserId,
        year: u16,
        month: u8,
        day: u8,
        is_working: bool,
    ) -> Response {
        match self
            .database
            .set_schedule(
                &ScheduleData {
                    day: day as i32,
                    month: month as i32,
                    year: year as i32,
                    user_id,
                },
                is_working,
            )
            .await
        {
            Ok(_) => self.get_schedule(year, month).await,
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn set_password(
        &self,
        user: UserData,
        old_password: String,
        new_password: String,
    ) -> Response {
        let mut user = user;
        if user.pwd_hash != user.get_pwd_hash(old_password) {
            return Err(ProtocolError::LoginFailed);
        }
        user.pwd_salt = utils::make_uuid();
        user.pwd_hash = user.get_pwd_hash(new_password);
        match self.database.update_user(&user).await {
            Ok(_) => Ok(ResponseData::PasswordChanged),
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn get_user_names(&self, ids: impl AsRef<[UserId]>) -> Response {
        match self.database.get_users(Some(ids.as_ref())).await {
            Ok(users) => Ok(ResponseData::UserNames {
                names: users.into_iter().map(|u| (u.id, u.name)).collect(),
            }),
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn get_users(&self) -> Response {
        match self.database.get_users(None).await {
            Ok(users) => Ok(ResponseData::Users(
                users
                    .into_iter()
                    .map(|u| User {
                        id: u.id,
                        login: u.login,
                        name: u.name,
                        is_admin: u.is_admin,
                        is_worker: u.is_worker,
                        pay: u.pay,
                        percent: u.percent,
                    })
                    .collect(),
            )),
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn add_user(&self, user: User) -> Response {
        if let Ok(Some(_)) = self
            .database
            .get_user(&UserSearch::Login(user.login.clone()))
            .await
        {
            return Err(ProtocolError::UserExist);
        }
        let mut user = UserData {
            id: 0,
            login: user.login,
            name: user.name,
            is_admin: user.is_admin,
            is_worker: user.is_worker,
            pay: user.pay,
            percent: user.percent,
            pwd_hash: "".to_string(),
            pwd_salt: utils::make_uuid(),
            token: utils::make_uuid(),
        };
        user.pwd_hash = user.get_pwd_hash("Qwer4321");
        match self.database.add_user(&user).await {
            Ok(_) => self.get_users().await,
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn reset_password(&self, id: UserId) -> Response {
        if let Ok(Some(mut user)) = self.database.get_user(&UserSearch::Id(id)).await {
            user.pwd_salt = utils::make_uuid();
            user.pwd_hash = user.get_pwd_hash("Qwer4321");
            user.token = utils::make_uuid();
            match self.database.update_user(&user).await {
                Ok(_) => Ok(ResponseData::PasswordReset),
                Err(e) => Err(ProtocolError::Unknown(e.to_string())),
            }
        } else {
            Err(ProtocolError::Unknown(
                "Не удалось найти пользователя".to_string(),
            ))
        }
    }

    async fn update_user(&self, new_user: User) -> Response {
        if let Ok(Some(mut user)) = self.database.get_user(&UserSearch::Id(new_user.id)).await {
            user.name = new_user.name;
            user.is_worker = new_user.is_worker;
            user.is_admin = new_user.is_admin;
            user.pay = new_user.pay;
            user.percent = new_user.percent;
            match self.database.update_user(&user).await {
                Ok(_) => self.get_users().await,
                Err(e) => Err(ProtocolError::Unknown(e.to_string())),
            }
        } else {
            Err(ProtocolError::Unknown(
                "Не удалось найти пользователя".to_string(),
            ))
        }
    }

    async fn get_revenue(&self, year: u16, month: u8) -> Response {
        match self.database.get_revenue(month, year).await {
            Ok(revenue) => Ok(ResponseData::Revenue {
                year,
                month,
                revenue: revenue
                    .into_iter()
                    .map(|r| Revenue {
                        day: r.day as u8,
                        with_percent: r.with_percent,
                        without_percent: r.without_percent,
                    })
                    .collect(),
            }),
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn set_revenue(&self, year: u16, month: u8, revenue: Revenue) -> Response {
        match self
            .database
            .set_revenue(&RevenueData {
                day: revenue.day as i32,
                month: month as i32,
                year: year as i32,
                with_percent: revenue.with_percent,
                without_percent: revenue.without_percent,
            })
            .await
        {
            Ok(_) => self.get_revenue(year, month).await,
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }

    async fn get_salary_calculation(&self, year: u16, month: u8) -> Response {
        match self.database.get_salaries(month, year).await {
            Ok(salaries) => Ok(ResponseData::SalaryCalculation {
                salaries: salaries
                    .into_iter()
                    .map(|s| Salary {
                        id: s.user_id,
                        total: s.amount_owed + s.amount_paid,
                        paid: s.amount_paid,
                    })
                    .collect(),
            }),
            Err(e) => Err(ProtocolError::Unknown(e.to_string())),
        }
    }
}
