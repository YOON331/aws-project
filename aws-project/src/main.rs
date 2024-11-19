use std::io::{self, Write};

use aws_sdk_ec2 as ec2;
use aws_config;

#[::tokio::main]
async fn main() -> Result<(), ec2::Error> {
    // AWS configuration 파일 읽어오기
    let config = aws_config::from_env()
        .profile_name("root-access") // AWS CLI의 특정 profile name의 설정값
        .region("us-east-1")
        .load()
        .await;

    // EC2 client 생성
    let client = ec2::Client::new(&config);


    loop {
        print_menu();

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).expect("failed to read line");
        let user_input = user_input.trim();
        
        match user_input {
            "99" => {
                println!("quit program");
                break
            },
            "1" => {
                println!("\n1. Listing instances....");
                // EC2 instances 리스트 
                let response = client.describe_instances().send().await?;
                
                // 응답된 리스트 매핑하여 출력
                for reservation in response.reservations() {
                    for instance in reservation.instances() {
                        println!("[ID] {}", instance.instance_id().unwrap());
                        println!("[AMI] {}", instance.image_id().unwrap());
                        println!("[Type] {}", instance.instance_type().unwrap());
                        println!("[State] {}", instance.state().map(|state| state.name()).unwrap().unwrap());
                        println!("[monitiring state] {}", instance.monitoring().unwrap().state().unwrap());
                        println!();
                    }
                }
            },
            "2" => {
                println!("\n2. Available zones....");

                let response = client.describe_availability_zones().send().await?;
                let r = response.availability_zones.unwrap();
                let available_cnt = r.len();
                for val in r {
                    println!("[ID] {}, [region] {}, [zone] {}",val.zone_id().unwrap(), val.region_name().unwrap(), val.zone_name().unwrap());
                }
                println!("You have access to {} Availability Zones.", available_cnt);
            },
            "3" => {
                print!("Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin().read_line(&mut instance_id).expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.start_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        for r in val.starting_instances.unwrap() {
                            println!("\nSuccessfully started [instance ID] {}", r.instance_id.unwrap());
                        }
                    }
                    Err(_) => println!("\nInvalid instance ID entered.\nPlease check the instance ID."),
                }
            },
            "4" => {
                
            },
            "5" => {
                print!("Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin().read_line(&mut instance_id).expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.stop_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        for r in val.stopping_instances.unwrap() {
                            println!("\nSuccessfully stop [instance ID] {}", r.instance_id.unwrap());
                        }
                    }
                    Err(_) => println!("\nInvalid instance ID entered.\nPlease check the instance ID."),
                }
            },
            "6" => {

            },
            "7" => {

            },
            "8" => {
            },
            _ => println!("Wrong input")
        }
    }
    Ok(())
}

fn print_menu() {
    println!("                                                            ");
    println!("                                                            ");
    println!("------------------------------------------------------------");
    println!("           Amazon AWS Control Panel using SDK               ");
    println!("------------------------------------------------------------");
    println!("  1. list instance                2. available zones        ");
    println!("  3. start instance               4. available regions      ");
    println!("  5. stop instance                6. create instance        ");
    println!("  7. reboot instance              8. list images            ");
    println!("                                 99. quit                   ");
    println!("------------------------------------------------------------");   
    print!("Enter an integer: ");
    let _ = io::stdout().flush();
}
