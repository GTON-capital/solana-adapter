

trait AbstractGravityContract {
    fn update_consuls(&self)
    fn hash_new_consuls(&self)
}

struct GravityContract {
    pub consuls: Vec<String>,

    pub bft_value: i32,
    pub last_round: i64
}



