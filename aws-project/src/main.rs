use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use aws_sdk_ec2::types::IamInstanceProfileSpecification;
use chrono::Datelike;
use aws_config;
use aws_sdk_ec2::{self as ec2, types::InstanceType};
use aws_sdk_ssm::Client as SsmClient;
use aws_sdk_cloudwatch::Client as CWClient;
use aws_sdk_cloudwatch::types::{Dimension, Statistic};
use aws_sdk_ec2::primitives::DateTime;
use aws_sdk_costexplorer::types::{DateInterval, GroupDefinition};

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

                let iam_instance_profile = IamInstanceProfileSpecification::builder()
                    .name("EC2SSMRole") // 인스턴스 프로파일 이름
                    .build();
                
                let request = client
                    .run_instances()
                    .image_id(ami_id)
                    .instance_type(InstanceType::T2Micro)
                    .max_count(1)
                    .min_count(1)
                    .iam_instance_profile(iam_instance_profile)
                    .key_name("cloud-yoon".to_string());
                let response = request.send().await;

                match response {
                    Ok(val) => {
                        let mut new_id:Vec<String> = val.instances.unwrap().iter().map(
                            |element| element.instance_id.clone().unwrap(),
                        ).collect();
                        println!(
                            "\nSuccessfully started EC2 instance {} based on AMI {}",
                            new_id.pop().unwrap(), ami_id
                        );
                    }
                    Err(e) => println!(
                        "\nInvalid AMI ID entered.\nPlease check the AMI ID.\n{}",
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
            "10" => {
                print!("10. terminate instance\nEnter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                let request = client.terminate_instances().instance_ids(instance_id);
                let response = request.send().await;

                match response {
                    Ok(_) => {
                        println!("\nSuccessfully terminated [instance ID] {}", instance_id);
                    }
                    Err(e) => println!(
                        "\nInvalid instance ID entered.\nPlease check the instance ID.\n{}",
                        e
                    ),
                }
            }
            "11" => {
                let cw_client = CWClient::new(&config);
                let now = SystemTime::now();
                let start_time = DateTime::from_secs(
                    (now.duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600).try_into().unwrap(),
                );
                let end_time = DateTime::from_secs(
                    now.duration_since(UNIX_EPOCH).unwrap().as_secs().try_into().unwrap(),
                );

                print!("Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();
                let namespace = "AWS/EC2";
                let metric_name = "CPUUtilization";

                match cw_client
                    .get_metric_statistics()
                    .namespace(namespace)
                    .metric_name(metric_name)
                    .start_time(start_time)
                    .end_time(end_time)
                    .period(60)
                    .statistics(Statistic::Average)
                    .dimensions(
                        Dimension::builder()
                            .name("InstanceId")
                            .value(instance_id)
                            .build(),
                    )
                    .unit("Percent".into())
                    .send()
                    .await
                {
                    Ok(response) => {
                        match response.datapoints {
                            Some(data) => {
                                if !data.is_empty() {
                                    println!("CPU Utilization Report");
                                    for point in data.iter() {
                                        if let Some(timestamp) = point.timestamp() {
                                            if let Some(average) = point.average() {
                                                println!(
                                                    "Time: {:?} | Average CPU: {:.2}%",
                                                    timestamp,
                                                    average
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    println!("No data points found for the specified metric.");
                                }
                                
                            }
                            _ => println!("No data points found for the specified metric."),
                        }
                    }
                    Err(e) => eprintln!("Failed to fetch CloudWatch metrics: {:?}", e),
                }
            }
            "12" => {
                let cost_client=aws_sdk_costexplorer::Client::new(&config);

                // 현재 월의 시작 날짜 계산
                let start_date = chrono::Utc::now()
                    .with_day(1)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string();
                let end_date = chrono::Utc::now()
                    .format("%Y-%m-%d")
                    .to_string();

                // 비용 조회 요청 생성
                match cost_client
                    .get_cost_and_usage()
                    .time_period(
                        DateInterval::builder()
                            .start(start_date)
                            .end(end_date)
                            .build().unwrap(),
                    )
                    .granularity("MONTHLY".into())
                    .metrics("BlendedCost")
                    .group_by(
                        GroupDefinition::builder()
                        .key("SERVICE")
                        .r#type("DIMENSION".into())
                        .build()
                    )   
                    .send()
                    .await
                {
                    Ok(response) => {
                        match response.results_by_time {
                            Some(result) => {
                                for val in result.iter() {
                                    println!(
                                        "Fetching cost details from {} to {}",
                                        val.time_period.clone().unwrap().start,
                                        val.time_period.clone().unwrap().end,
                                    );
                            
                                    if let Some(groups) = &val.groups {
                                        for group in groups {
                                            let service_name = group.keys.as_ref().map_or("Unknown Service".to_string(), |keys| keys.join(", "));
                                            if let Some(metrics) = &group.metrics {
                                                if let Some(blended_cost) = metrics.get("BlendedCost") {
                                                    let amount = blended_cost.amount.clone().unwrap_or("0".to_string());
                                                    let unit = blended_cost.unit.clone().unwrap_or("USD".to_string());
                                                    println!("Service: {}, Cost: {} {}", service_name, amount, unit);
                                                }
                                            }
                                        }
                                    } else {
                                        println!("No group data found.");
                                    }
                            
                                    if let Some(total) = &val.total {
                                        for (metric_name, metric_value) in total {
                                            println!(
                                                "Total {}: {} {}",
                                                metric_name,
                                                metric_value.amount.clone().unwrap_or("0".to_string()),
                                                metric_value.unit.clone().unwrap_or("USD".to_string())
                                            );
                                        }
                                    }
                                    println!("Estimated: {}", val.estimated);
                                }
                            },
                            
                            None => println!("No data points found for the specified metric."),
                        }
                    }
                    Err(e) => println!("Failed to fetch cost details: {}", e),
                }
            }
            "13" => {
                print!("13. Enter instance id: ");
                let _ = io::stdout().flush();
                let mut instance_id = String::new();
                io::stdin()
                    .read_line(&mut instance_id)
                    .expect("failed to read line");
                let instance_id = instance_id.trim();

                print!("Enter command-line: ");
                let _ = io::stdout().flush();
                let mut command_line = String::new();
                io::stdin()
                    .read_line(&mut command_line)
                    .expect("failed to read line");
                

                let ssm_client = SsmClient::new(&config);
                let op = vec![command_line.clone()];
                // Shell script 명령 실행 요청
                let response = ssm_client
                    .send_command()
                    .document_name("AWS-RunShellScript") // AWS에서 제공하는 기본 문서
                    .instance_ids(instance_id.to_string())
                    .parameters("commands", op)
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
                            "Failed to execute {} command on instance {}: {:?}",
                            command_line,instance_id, e
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
    println!("  9. condor_status               10. terminate instance     ");
    println!(" 11. CPU Utilization             12. Cost Analysis          ");
    println!(" 13. Run Shell Command                                      ");
    println!("                                 99. quit                   ");
    println!("------------------------------------------------------------");
    print!("Enter an integer: ");
    let _ = io::stdout().flush();
}
