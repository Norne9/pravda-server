use crate::database::*;
use pravda_protocol::*;
use uuid::Uuid;

pub struct PravdaHandler<T>
where
    T: Database,
{
    database: T,
}

impl<T: Database> PravdaHandler<T> {
    pub fn new(database: T) -> Self {
        Self { database }
    }

    pub async fn process(&self, request: Request, token: Option<String>) -> Response {
        match request {
            Request::Login { login, password } => return self.p_login(login, password).await,
            _ => (),
        };

        let user = match token {
            Some(token) => match self.database.get_user(&UserSearch::Token(token)).await {
                Ok(user) => user,
                Err(_) => return Err(ProtocolError::UnknownToken),
            },
            None => return Err(ProtocolError::Forbidden),
        };

        return match request {
            Request::Login { .. } => panic!("Can't be login here!"),
            Request::GetUserInfo => Ok(ResponseData::UserInfo(User {
                id: user.id,
                login: user.login,
                name: user.name,
                is_admin: user.is_admin,
                is_worker: user.is_worker,
                pay: user.pay,
                percent: user.percent,
            })),
            Request::GetSchedule { .. } => todo!(),
            Request::SetWorkday { .. } => todo!(),
            Request::ChangePassword { .. } => todo!(),
            Request::GetUserNames { .. } => todo!(),
            Request::GetUsers => todo!(),
            Request::AddUser(_) => todo!(),
            Request::ResetPassword { .. } => todo!(),
            Request::UpdateUser(_) => todo!(),
            Request::GetRevenue { .. } => todo!(),
            Request::SetRevenue { .. } => todo!(),
            Request::GetSalaryCalculation { .. } => todo!(),
        };
    }

    async fn p_login(&self, login: String, password: String) -> Response {
        let mut user = match self.database.get_user(&UserSearch::Login(login)).await {
            Ok(user) => user,
            Err(_) => return Err(ProtocolError::LoginFailed),
        };
        if user.pwd_hash == user.get_pwd_hash(password) {
            let token = Uuid::new_v4().to_string();
            user.token = token.clone();
            match self.database.update_user(&user).await {
                Ok(user) => Ok(ResponseData::Login {
                    token: user.token,
                    id: user.id,
                }),
                Err(_) => Err(ProtocolError::Unknown),
            }
        } else {
            Err(ProtocolError::LoginFailed)
        }
    }
}

impl UserData {
    fn get_pwd_hash(&self, password: impl AsRef<str>) -> String {
        use sha3::{Digest, Sha3_256};

        let pwd = format!("{}#{}", password.as_ref(), self.pwd_salt);

        let mut hasher = Sha3_256::new();
        hasher.update(pwd.as_bytes());
        format!("{:x?}", hasher.finalize())
    }
}
