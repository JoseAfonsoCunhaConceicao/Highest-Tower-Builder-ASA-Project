use rand::RngExt;

fn main() {
     loop {
          let mut rng = rand::rng();
          let random_number: u32 = rng.random_range(0..200);
          println!("{}", random_number);
          if random_number > 180 {
               println!("Eu sou lindo");
               break;
          }
          println!("És feio que doi");
     }
}