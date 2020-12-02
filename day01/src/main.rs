use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut amounts: Vec<isize> = Vec::new();
    for line in std::io::stdin().lock().lines() {
        amounts.push(line?.trim().parse::<isize>()?);
    }
    amounts.sort();
    
    let mut lft = 0;
    let mut rgt = amounts.len()-1;
    while lft < rgt {
        let sum = amounts[lft] + amounts[rgt];
        if sum == 2020 {
            println!("{}", amounts[lft] * amounts[rgt]);
            break;
        }
        else if sum < 2020 {
            lft += 1;
        }
        else if sum > 2020 {
            rgt -= 1;
        }
    }
    Ok(())
}
