
pub fn operational_fee() -> f64 {
    // 0.1 %
    0.1
}

fn calc_fee(input: f64) -> f64 {
    let percentage: f64 = 100.0;
    operational_fee() * input / percentage
}
pub fn apply_fee(input: f64) -> (f64, f64) {
    // let percentage: f64 = 100.0;
    // let fee = operational_fee() * input / percentage;
    let fee = calc_fee(input);
    (input - fee, fee)
}

pub fn apply_fee_lamports(input: u64, decimals: u8) -> (u64, u64) {
    let casted = lamports_to_float(input, decimals);
    let (casted_amount, casted_fee) = apply_fee(casted);
    (float_to_lamports(casted_amount, decimals), float_to_lamports(casted_fee, decimals))
}

fn lamports_to_float(input: u64, decimals: u8) -> f64 {
    let qtr = (10 as f64).powf(decimals as f64);
    let casted = (input as f64) / qtr;
    casted
}
fn float_to_lamports(input: f64, decimals: u8) -> u64 {
    let qtr = (10 as u64).pow(decimals as u32) as f64;
    let casted = (input as f64) * qtr;
    casted as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with_values(amount: f64, decimals: u8, fee_taken: f64) {
        let base: f64 = 10.0;
        let (amount_lamports, lamports_fee_taken) = (
            (amount * base.powf(decimals as f64)) as u64,
            (fee_taken * base.powf(decimals as f64)) as u64,
        );

        assert_eq!(amount_lamports, float_to_lamports(amount, decimals));
        assert_eq!(lamports_fee_taken, float_to_lamports(fee_taken, decimals));

        let (fee_taken_calc, fee_calc) = apply_fee(amount);
        let (fee_taken_calc_lamports, fee_calc_lamports) = apply_fee_lamports(amount_lamports, decimals);

        assert_eq!(fee_taken, fee_taken_calc);
        assert_eq!(lamports_fee_taken, fee_taken_calc_lamports);

        assert_eq!(calc_fee(amount), fee_calc);
        assert_eq!(float_to_lamports(calc_fee(lamports_to_float(amount_lamports, decimals)), decimals), fee_calc_lamports);
    }

    struct Input {
        amount: f64,
        fee_taken: f64,
        decimals: u8,
    }

    #[test]
    fn check_calculation() {
        let input_values: Vec<Input> = vec![
            Input { 
                amount: 100_000_000.0,
                decimals: 8,
                fee_taken: 100_000_000.0 - 100_000.0
            },
            Input { 
                amount: 1_000_000_000.0,
                decimals: 8,
                fee_taken: 1_000_000_000.0 - 1_000_000.0
            },
            Input { 
                amount: 0.345987,
                decimals: 8,
                fee_taken: 0.345987 - 0.000345987
            },
            Input { 
                amount: 100_000_000.0,
                decimals: 3,
                fee_taken: 100_000_000.0 - 100_000.0
            },
            Input { 
                amount: 1_000_000_000.0,
                decimals: 3,
                fee_taken: 1_000_000_000.0 - 1_000_000.0
            },
            Input { 
                amount: 0.345987,
                decimals: 6,
                fee_taken: 0.345987 - 0.000345987
            },
            Input { 
                amount: 1.0,
                decimals: 4,
                fee_taken: 1.0 - (1.0 * 0.1 / 100.0),
            },
        ];

        input_values.iter().for_each(|x| {
            test_with_values(x.amount, x.decimals, x.fee_taken);
        });
    }
}