use axum::{
    extract::Path,
    routing::{get, post, delete, put},
    Router,
};
use rand::prelude::*;
use std::fs::{self, read_to_string, remove_dir_all};
use std::{fs::{create_dir, read_dir, File, OpenOptions}, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    password: String,
    token: String
}
#[derive(Serialize, Deserialize, Debug)]
struct Repository {
    name: String,
    repo_type: String,
    archived: bool
}


#[tokio::main]
async fn main() {
    println!("Server is now running on: http://localhost:5023");
    let app = Router::new()
        .route("/", get("Hello world!"))
        .route(
            "/create_repo/:user_token/:name/:user_name/:repo_type",
            post(create_repo),
        )
        .route(
            "/delete_repo/:user_token/:name/:user_name",
            delete(delete_repo),
        )
        .route(
            "/list_repo/:user_token/:user_name",
            get(list_repo),
        )
        .route(
            "/rename_repo/:user_token/:user_name/:repo_name/:new_name",
            put(rename_repo),
        )
        .route(
            "/archive_repo/:user_token/:user_name/:name",
            put(archive_repo),
        )
        .route(
            "/restore_repo/:user_token/:user_name/:name",
            put(restore_repo),
        )
        .route(
            "/create_branch/:user_token/:user_name/:name/:repo_name",
            post(create_branch),
        )
        .route(
            "/delete_branch/:user_token/:user_name/:name/:repo_name",
            post(delete_branch),
        )
        .route(
            "/list_branch/:user_token/:user_name/:repo_name",
            get(list_branch),
        )
        .route(
            "/add_file/:user_token/:user_name/:name/:contents/:repo_name/:branch_name",
            post(add_file),
        )
        .route(
            "/remove_file/:user_token/:user_name/:name/:repo_name/:branch_name",
            delete(remove_file),
        )
        .route(
            "/update_file/:user_token/:user_name/:name/:contents/:repo_name/:branch_name",
            put(update_file),
        )
        .route(
            "/list_file/:user_token/:user_name/:repo_name/:branch_name",
            get(list_files),
        )
        .route(
            "/view_file/:user_token/:user_name/:name/:repo_name/:branch_name",
            get(view_file),
        )
        .route(
            "/sign_up/:user/:pwd",
            post(signup),
        )
        .route(
            "/login/:user/:pwd",
            get(login),
        );
    axum::Server::bind(&"0.0.0.0:5023".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn create_repo(Path((user_token, user_name, name, repo_type)): Path<(String, String, String, String)>) -> String {

    if auth_user(&user_name, &user_token) {
        let main_branch = format!("main_data/{}/{}", user_name, name);
        let _branch_dir = fs::create_dir(main_branch).expect("Failed to create branch!");
        let location = format!("main_data/{}/{}/main", user_name, name);
        let config_location = format!("main_data/{}/{}", user_name, name);
        let _repo_dir = fs::create_dir(&location).expect("Failed to create directory!");
        
        let config_contents = format!("{{\"name\": \"{}\",\n\"repo_type\": \"{}\",\n\"archived\": false}}", name, repo_type);
        let _config_file = fs::write(config_location + "/.json", config_contents);
        let default_file = fs::write(location + "/index.md", "<h1>Hello, World!</h1>");
        match default_file {
            Ok(_) => {
                return format!("Successfully created repository!\nName: {}\nrepo_type: {}", name, repo_type);
            },
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
        
    } else {
        return "There was an error".to_string();
    }

    
}

pub async fn delete_repo(Path((user_token, user_name, name)): Path<(String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let location = format!("main_data/{}/{}", user_name, name);
        let remove_repo = fs::remove_dir_all(&location);

        match remove_repo {
            Ok(_) => {
                return format!("{} has been deleted successfully!", name);
            }
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    }
}

pub async fn list_repo(Path((user_token, user_name)): Path<(String, String)>) -> String {

    if auth_user(&user_name, &user_token) {
        let directory_path = format!("main_data/{}", user_name);
        let users = read_dir(directory_path).unwrap();
        let mut repos = String::from("");
        for path in users {
            let file_name = format!("{}", path.unwrap().path().display().to_string());
            let removing_string = format!(r"main_data/{}\", user_name);
            if removing_string.clone() + ".json" != file_name {
                println!("{}", format!("{} {}", file_name, removing_string.clone() + ".json"));
                repos.push_str(&file_name.replace(&removing_string, "\n"));
            }
            println!("{}", removing_string);
        }

        return repos;
    } else {
        return "There was an error".to_string();
    }
}

async fn rename_repo(Path((user_token, user_name, repo_name, new_name)): Path<(String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let location = format!("main_data/{}/", user_name);

        let result = std::fs::rename(format!("{}{}", location, repo_name), format!("{}{}", location, new_name));
        match result {
            Ok(_) => {
                return format!("{} has been renamed to {}", repo_name, new_name);
            },
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    }
}

async fn archive_repo(Path((user_token, user_name, name)): Path<(String, String, String)>) -> String {

    if auth_user(&user_name, &user_token) {
        let file_location = format!("main_data/{}/{}/.json", user_name, name);
        let string_data = read_to_string(&file_location).unwrap();
        let mut repo_data: Repository = serde_json::from_str(&string_data).unwrap(); 
        
        repo_data.archived = true;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_location)
            .unwrap();


        let stringified_result = serde_json::to_string(&repo_data);

        match stringified_result {
            Ok(stringified_json) => {
                let result = file.write(stringified_json.as_bytes());
                match result {
                    Ok(_) => {
                        return "Successfully archived the repository!".to_string();
                    },
                    Err(e) => {
                        return format!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                return format!("{}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    }
}

async fn restore_repo(Path((user_token, user_name, name)): Path<(String, String, String)>) -> String {

    if auth_user(&user_name, &user_token) {
        let file_location = format!("main_data/{}/{}/.json", user_name, name);
        let string_data = read_to_string(&file_location).unwrap();
        let mut repo_data: Repository = serde_json::from_str(&string_data).unwrap(); 
        
        repo_data.archived = false;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_location)
            .unwrap();


        let stringified_result = serde_json::to_string(&repo_data);

        match stringified_result {
            Ok(stringified_json) => {
                let result = file.write(stringified_json.as_bytes());
                match result {
                    Ok(_) => {
                        return "Successfully restored the repository!".to_string();
                    },
                    Err(e) => {
                        return format!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                return format!("{}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    }
}

async fn add_file(Path((user_token, user_name, name, contents, repo_name , branch_name)): Path<(String, String, String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let file_path = format!("main_data/{}/{}/{}/{}", user_name, repo_name, branch_name, name);

        let file = fs::write(&file_path, contents);
        match file {
            Ok(_) => {
                return format!("{} has been added successfully to {} branch, {} repository!", name, branch_name, repo_name);
            },
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    };
}
async fn remove_file(Path((user_token, user_name, name, repo_name , branch_name)): Path<(String, String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let file_path = format!("main_data/{}/{}/{}/{}", user_name, repo_name, branch_name, name);

        let file = fs::remove_file(&file_path);

        match file {
            Ok(_) => {
                return format!("{} has been removed successfully to {} branch, {} repository!", name, branch_name, repo_name);
            },
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    };
}

async fn list_files(Path((user_token, user_name, repo_name , branch_name)): Path<(String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let directory_path = format!("main_data/{}/{}/{}", user_name, repo_name, branch_name);
        let users = read_dir(directory_path).unwrap();

        let mut files = String::from("");

        for path in users {
            let file_name = format!("{}", path.unwrap().path().display().to_string());
            let removing_string = format!(r"main_data/{}/{}\", user_name, repo_name);
            files.push_str(&file_name.replace(&removing_string, "\n"));
        };

        return files;
    } else {
        return "There was an error".to_string();
    };
}

async fn update_file(Path((user_token, user_name, name, contents, repo_name , branch_name)): Path<(String, String, String, String, String, String)>) -> String {
    let directory_path = format!("main_data/{}/.json", user_name);
    let user = read_to_string(directory_path);

    match user {
        Ok(txt) => {
            let jsonified_data: User = serde_json::from_str(txt.as_str()).unwrap();

            if jsonified_data.token == user_token && archive_check(&user_name, &repo_name) {
                let file_path = format!("main_data/{}/{}/{}/{}", user_name, repo_name, branch_name, name);

                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&file_path);
                

                match file {
                    Ok(mut f) => {
                        let result = f.write(contents.as_bytes());
                        match result {
                            Ok(_) => {
                                return "Successfully updated file!".to_string();
                            }
                            Err(e) => {
                                return format!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        return format!("Error: {}", e);
                    }
                }
            } else {
                return "Oops! Something went wrong, try again later!".to_string();
            };
        },
        Err(e) => {
            return format!("Error {}", e);
        }
    }
}

async fn view_file(Path((user_token, user_name, name, repo_name , branch_name)): Path<(String, String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {
        let file_path = format!("main_data/{}/{}/{}/{}", user_name, repo_name, branch_name, name).replace(" ", "");

        let file_data = read_to_string(file_path);
                
        match file_data {
            Ok(data) => {
                return data;
            }
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    };
}

async fn create_branch(Path((user_token, user_name, name, repo_name)): Path<(String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) && archive_check(&user_name, &repo_name) {
        let location = format!("main_data/{}/{}/{}", user_name, repo_name, name);

        let result = create_dir(&location);

        match result {
            Ok(_) => {
                let default_file = fs::write(format!("{}{}", &location, "/index.md"), "<h1>Hello, World!</h1>");
                match default_file {
                    Ok(_) => {
                        return format!("Successfully created branch!\nName: {}", name);
                    },
                    Err(e) => {
                        return format!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    }
}

async fn delete_branch(Path((user_token, user_name, name, repo_name)): Path<(String, String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) && archive_check(&user_name, &repo_name) {
        let location = format!("main_data/{}/{}/{}", user_name, repo_name, name);

        let result = create_dir(&location);

        match result {
            Ok(_) => {
                let location = format!("main_data/{}/{}/{}", user_name, repo_name, name);
                
                let result = remove_dir_all(&location);

                match result {
                    Ok(_) => {
                        return format!("Successfully removed {} branch!", name)
                    }
                    Err(e) => {
                        return format!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                return format!("Error: {}", e);
            }
        }
    } else {
        return "There was an error".to_string();
    };
}

async fn list_branch(Path((user_token, user_name, repo_name)): Path<(String, String, String)>) -> String {
    if auth_user(&user_name, &user_token) {

        let directory_path = format!("main_data/{}/{}", user_name, repo_name);
        let users = read_dir(directory_path).unwrap();
        let mut repos = String::from("");
        for path in users {
            let file_name = format!("{}", path.unwrap().path().display().to_string());
            let removing_string = format!(r"main_data/{}\", user_name);
            if removing_string.clone() + ".json" != file_name {
                println!("{}", format!("{} {}", file_name, removing_string.clone() + ".json"));
                repos.push_str(&file_name.replace(&removing_string, "\n"));
            }
        }
        return repos;
    } else {
        return "There was an error".to_string();
    };
}

fn auth_user(user_name: &String, user_token: &String) -> bool {
    let directory_path = format!("main_data/{}/.json", user_name);
    let user = read_to_string(directory_path);


    match user {
        Ok(txt) => {
            let jsonified_data: User = serde_json::from_str(txt.as_str()).unwrap();
            if &(jsonified_data.token) == user_token {
                return true;
            };
            return false;
        },
        Err(_) => {
            return false;
        }
    }
}

fn archive_check(user_name: &String, repo_name: &String) -> bool {
    let config_file_location = format!("main_data/{}/{}/.json", user_name, repo_name);

    let jsonified_data: Repository = serde_json::from_str(&config_file_location.as_str()).expect("JSON object conversion failed!");

    if !(jsonified_data.archived) {
        return true;
    }

    return false;
}

fn signup_setup(file_path: String, user_name: String, pwd: String) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
                .write(true)       // Enable writing
                .append(true)    // Truncate the file to 0 length
                .open(file_path)?; // Open the file

    let mut token: String = String::from("");

    let alphabet_list = ["a", "b", "c", "3", "*", ">", "t", "-", "2"];

    let mut rng = rand::thread_rng();

    for _ in 0..alphabet_list.len() {
        let i: usize = rng.gen_range(0..alphabet_list.len());
        token.push_str(alphabet_list[i]);
    }

    let file_content = format!("{{\"name\": \"{}\", \"password\": \"{}\",\n\"token\": \"{}\"}}", user_name, pwd, token);

    println!("{}", file_content);

    let _ = file.write_all(file_content.as_bytes());

    Ok(())
}

async fn signup(Path((user, pwd)): Path<(String, String)>) -> String {
    let user_path: String = format!("main_data/{}", user);
    let result = read_dir(&user_path);
    
    match result {
        Ok(_) => {
            return format!("Account already exists!")
        },
        Err(_) => {
            create_dir(&user_path).unwrap();
            
            let json_file = format!("{}/.json", user_path);
        
            let _password_file: File = File::create(json_file.clone()).unwrap();
                
            let setup = signup_setup(json_file, user, pwd);
                
            match setup {
                Ok(_) => "Account creation was successful!".to_string(),
                Err(e) => format!("Error: {}", e)
            }
        }
    }

    
}

async fn login(Path((name, pwd)): Path<(String, String)>) -> String {

    let directory_path = format!("main_data/{}", name);
    let users = read_dir(directory_path);

    match users {
        Ok(users) => {
            for path in users {
                let file_name = path.unwrap().path().display().to_string();
        
                let user_name = format!("main_data/{}{}", name, r"\.json");
        
                println!("{} {}", file_name, user_name);
                if file_name == user_name {
        
                    let file_path = format!("main_data/{}/.json", name);
                    let file_data = fs::read_to_string(file_path).unwrap();
                    let jsonified_data: User = serde_json::from_str(&file_data).unwrap();
        
                    if jsonified_data.password == pwd {
                        return file_data;
                    } else {
                        return "Failed to authorize user!".to_string();
                    }
                }
            };
        },
        Err(err) => {
            return err.to_string();
        }
    }

    "Failed to authorize user!".to_string()
}