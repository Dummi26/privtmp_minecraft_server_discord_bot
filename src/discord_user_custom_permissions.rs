use serenity::model::user::User;

const DISCORD_ID_OWNER: u64 = 402830098870435857;

pub struct DcUserPerms {
    user: User,
    admin: Option<bool>,
}
impl DcUserPerms {
    pub fn new(user: User) -> Self {
        Self {
            user: user,
            admin: None,
        }
    }

    // - - - - -

    pub fn get_user(&self) -> &User { &self.user }

    pub fn is_admin(&mut self) -> bool {
        match self.admin {
            None => {
                let v = self.user.id == DISCORD_ID_OWNER;
                self.admin = Some(v);
                v
            },
            Some(v) => v
        }
    }
}