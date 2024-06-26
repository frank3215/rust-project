use std::fs;
use std::io::Write;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml::Value;

fn main() {
    let grade_file = "1.txt";
    let policy_file = "politics.txt";
    // let output_file = "output.txt";

    // Read the contents of the input file, and convert all whitespace characters to spaces
    let grade_contents = fs::read_to_string(grade_file).expect("Failed to read input file");
    let grade_contents = grade_contents.split_whitespace().collect::<Vec<_>>().join(" ");
    let grade_contents = grade_contents.replace(")","）").replace("(", "（").replace(" （I", "（I").replace(" （A", "（A").replace("I ）", "I）");
    // println!("{grade_contents}");

    let policy_contents = fs::read_to_string(policy_file).expect("Failed to read input file");

    // let input = "k: 107\n";
    let de = serde_yaml::Deserializer::from_str(&policy_contents);
    let course_policy = Value::deserialize(de).unwrap();
    // println!("{:?}", value);
    // println!("{:?}", value["公共必修课"][0]["name"]);

    // 3 学分 计算理论导论 专业必修 - 程宽（计算机学院） 87.3 3.70
    let re = Regex::new(r"(?<credit>\d) 学分 (?<course>\S+) (?<course_kind>\S+) - (?<teacher>\S+) (?<grade>100|[6-9][0-9]|[6-9][0-9]\.[0-9]|P) (?<gpa>.\...)").unwrap();

    // Write the modified content to the output file
    // fs::write(output_file, output_contents).expect("Failed to write output file");

    let mut total_credit = 0;
    let mut respective_credit: HashMap<String, i32> = HashMap::new();
    let mut course_credit: HashMap<String, i32> = HashMap::new();
    let mut course_taken = HashSet::new();
    let mut course_counted: HashSet<String> = HashSet::new();

    for capture in re.captures_iter(&grade_contents) {
        let credit: i32 = capture["credit"].parse().unwrap();
        total_credit += credit;
        let course = (&capture["course"]).to_string().replace("（实验班）", "").replace("（", "").replace("）", "");
        course_taken.insert(course.to_string());
        course_credit.insert(course.to_string(), credit);
        let course_kind = &capture["course_kind"];
        respective_credit.entry(course_kind.to_string()).and_modify(|c| *c += credit).or_insert(credit);
        let teacher = &capture["teacher"];
        let grade = &capture["grade"];
        let gpa = &capture["gpa"];
        // println!("{credit}, {course}, {course_kind}, {teacher}, {grade}, {gpa}");
    }
    // println!("{course_taken:?}");

    println!("已修 {total_credit} 学分，距修完约剩下 {} 学分", 150 - total_credit);

    for (course_kind, kind_credit) in &respective_credit {
        if course_kind == "通选课" {
            println!("通选课已修 {} 学分，距修完还剩下 {} 学分", kind_credit, 12 - kind_credit);
        } else {
            println!("{course_kind} 课已修 {kind_credit} 学分");
        }
    }

    let Value::Mapping(course_policy) = course_policy else { panic!() };
    for (course_kind, course_requirements) in course_policy {
        let Value::String(course_kind) = course_kind
            else { panic!() };
        println!("{course_kind}:");
        let Value::Sequence(course_requirements) = course_requirements
            else { panic!() };
        for course_requirement in &course_requirements {
            // println!("\t{course_requirement:?}")
            let Value::Mapping(course_requirement) = course_requirement
                else { panic!() };
            let Value::String(course_name) = &course_requirement["name"]
                else { panic!() };
            let course_name = course_name.replace("（", "").replace("）", "");
            println!("\t{course_name}");
            if course_requirement.contains_key("type") {
                let Value::String(course_type) = &course_requirement["type"]
                    else { panic!() };
                if course_type == "必修课" {
                    if !course_taken.contains(&course_name) {
                        println!("\t\t还没学");
                    } else {
                        course_counted.insert(course_name.to_string());
                        println!("\t\t已学");
                    }
                } else if course_type == "必修" {
                    if !course_requirement.contains_key("credit") && !course_requirement.contains_key("count") {
                        println!("\t\t全部必修，下面仅仅列出还没学的课程");
                        let Value::Sequence(course_list) = &course_requirement["course"]
                            else { panic!() };
                        for sub_course in course_list {
                            let Value::String(course_name) = sub_course
                                else { panic!() };
                            let course_name = course_name.replace("（", "").replace("）", "");
                            if !course_taken.contains(&course_name) {
                                println!("\t\t{course_name} 还没学");
                            } else {
                                course_counted.insert(course_name.to_string());
                                // println!("\t\t{course_name} 已学");
                            }
                        }
                    } else if course_requirement.contains_key("credit") {
                        let Value::Number(credit_requirement) = &course_requirement["credit"]
                            else { panic!() };
                        println!("\t\t必修 {credit_requirement} 学分");
                        let mut credit_count = 0;
                        if !course_requirement.contains_key("course") {
                            continue;
                        }
                        let Value::Sequence(course_list) = &course_requirement["course"]
                            else { panic!() };
                        for sub_course in course_list {
                            let Value::String(course_name) = sub_course
                                else { panic!() };
                            let course_name = course_name.replace("（", "").replace("）", "");
                            if !course_taken.contains(&course_name) {
                                // println!("\t\t{course_name} 还没学");
                            } else {
                                println!("\t\t{course_name} 已学");
                                course_counted.insert(course_name.to_string());
                                credit_count += course_credit[&course_name];
                            }
                        }
                        println!("\t\t一共学了 {credit_count}/{credit_requirement} 学分")
                    }
                } else if course_type == "选修" {
                    println!("\t\t此部分为选修，详情参考培养方案");
                    if !course_requirement.contains_key("credit") && !course_requirement.contains_key("count") {
                        let mut credit_count = 0;
                        let Value::Sequence(course_list) = &course_requirement["course"]
                            else { panic!() };
                        for sub_course in course_list {
                            let Value::String(course_name) = sub_course
                                else { panic!() };
                            let course_name = course_name.replace("（", "").replace("）", "");
                            if !course_taken.contains(&course_name) {
                                // println!("\t\t{course_name} 还没学");
                            } else {
                                println!("\t\t{course_name} 已学");
                                course_counted.insert(course_name.to_string());
                                credit_count += course_credit[&course_name];
                            }
                        }
                        println!("\t\t一共学了 {credit_count} 学分")
                    }
                }
            }
            // if course_requirement

        }
    }

    let mut remaining_credit = 0;
    for course in course_taken {
        if !course_counted.contains(&course) {
            println!("\t\t{course} 已学");
            remaining_credit += course_credit[&course];
        }
    }
    println!("\t\t一共学了 {remaining_credit} 学分");
}
