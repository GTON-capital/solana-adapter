
trait AbstractGravityContract {
    type Consul;

    fn get_consuls() -> Self::Consul;
    fn update_consuls(&self);
    fn hash_new_consuls(&self);
}

struct GravityContract {
    pub consuls: Vec<String>,
    pub bft_value: i32,
    pub last_round: i64
}



