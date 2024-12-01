use std::io::{self, Write};

use aws_config;
use aws_sdk_ec2::{self as ec2, types::InstanceType};
use aws_sdk_ssm::{self as ssm, Client as SsmClient};

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
        io::stdin()
            .read_line(&mut user_input)
            .expect("failed to read line");
        let user_input = user_input.trim();

        match user_input {
            "99" => {
                println!("quit program");
                break;
            }
            "1" => {
                println!("1. Listing instances....");
                // EC2 instances 리스트
                let response = client.describe_instances().send().await?;

                // 응답된 리스트 매핑하여 출력
                for reservation in response.reservations() {
                    for instance in reservation.instances() {
                        println!("[ID] {}", instance.instance_id().unwrap());
                        println!("[AMI] {}", instance.image_id().unwrap());
                        println!("[Type] {}", instance.instance_type().unwrap());
                        println!(
                            "[State] {}",
                            instance.state().map(|state| state.name()).unwrap().unwrap()
                        );
                        println!(
                            "[monitiring state] {}",
                            instance.monitoring().unwrap().state().unwrap()
                        );
                        println!();
                    }
                }
            }
            "2" => {
                println!("2. Available zones....");

                let response = client.describe_availability_zones().send().await?;
                let r = response.availability_zones.unwrap();
                let available_cnt = r.len();
                for val in r {
                    println!(
                        "[ID] {} [region] {: <15 } [zone] {}",
                        val.zone_id().unwrap(),
                        val.region_name().unwrap(),
                        val.zone_name().unwrap()
                    );
                }
                println!("You have access to {} Availability Zones.", available_cnt);
            }
            "3" => {
                print!("3. Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.start_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        for r in val.starting_instances.unwrap() {
                            println!(
                                "\nSuccessfully started [instance ID] {}",
                                r.instance_id.unwrap()
                            );
                        }
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "4" => {
                println!("4. Available regions....");

                let response = client.describe_regions().send().await;

                match response {
                    Ok(val) => {
                        for r in val.regions() {
                            println!(
                                "[region] {: <15} [endpoint] {}",
                                r.region_name.clone().unwrap(),
                                r.endpoint.clone().unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
            "5" => {
                print!("5. Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.stop_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        for r in val.stopping_instances.unwrap() {
                            println!(
                                "\nSuccessfully stopped [instance ID] {}",
                                r.instance_id.unwrap()
                            );
                        }
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "6" => {
                print!("6. Enter AMI id: ");
                let _ = io::stdout().flush();
                let mut ami_id = String::new();
                io::stdin()
                    .read_line(&mut ami_id)
                    .expect("failed to read line");
                let ami_id = ami_id.trim();

                let request = client
                    .run_instances()
                    .image_id(ami_id)
                    .instance_type(InstanceType::T2Micro)
                    .max_count(1)
                    .min_count(1);
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        println!(
                            "\nSuccessfully started EC2 instance {} based on AMI {}",
                            val.reservation_id.unwrap(),
                            ami_id
                        );
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "7" => {
                print!("7. Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.reboot_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(_) => {
                        println!("\nSuccessfully rebooted [instance ID] {}", instance_id);
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "8" => {
                println!("8. Listing images....");
                // owners id로 조회
                let request = client.describe_images().owners("509399609684");
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        let v = val.images.unwrap().into_iter();
                        for r in v {
                            println!(
                                "[ImageID] {} [Name] {} [Owner] {}",
                                r.image_id.unwrap_or("None".to_string()),
                                r.name.unwrap_or("None".to_string()),
                                r.owner_id.unwrap_or("None".to_string())
                            );
                        }
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "9" => {
                print!("9. Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                let ssm_client = SsmClient::new(&config);

                // `condor_status` 명령 실행 요청
                let response = ssm_client
                    .send_command()
                    .document_name("AWS-RunShellScript") // AWS에서 제공하는 기본 문서
                    .instance_ids(instance_id.to_string())
                    .parameters("commands", vec!["condor_status".to_string()])
                    .send()
                    .await;
                match response {
                    Ok(command_response) => {
                        let command_id = command_response
                            .command()
                            .and_then(|cmd| cmd.command_id())
                            .expect("Failed to retrieve command ID");
                
                        println!("Command ID: {}", command_id);
                
                        // 명령 실행 결과 가져오기
                        loop {
                            let result = ssm_client
                                .get_command_invocation()
                                .command_id(command_id)
                                .instance_id(&*instance_id)
                                .send()
                                .await;
                
                            match result {
                                Ok(invocation_response) => {
                                    if let Some(status) = invocation_response.status() {
                                        match status.as_str() {
                                            "InProgress" | "Pending" => {
                                                println!("Command is still running...");
                                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                                continue;
                                            }
                                            "Success" => {
                                                if let Some(output) = invocation_response.standard_output_content() {
                                                    println!("Command Output:\n{}", output);
                                                } else {
                                                    println!("No output from command.");
                                                }
                                                break;
                                            }
                                            _ => {
                                                eprintln!("Command failed with status: {}", status);
                                                if let Some(error) = invocation_response.standard_error_content() {
                                                    eprintln!("Error Output:\n{}", error);
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to fetch command output: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "Failed to execute `condor_status` command on instance {}: {:?}",
                            instance_id, e
                        );
                    }
                }
                
            }
            _ => println!("Wrong input"),
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
    println!("  9. condor_status                                          ");
    println!("                                 99. quit                   ");
    println!("------------------------------------------------------------");
    print!("Enter an integer: ");
    let _ = io::stdout().flush();
}
