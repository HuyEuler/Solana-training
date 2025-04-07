use std::io;

fn main(){
    let user1 = User {
        username : String::from("Huy Le"),
        password : String::from("12345"),
        age : 22
    };
    println!("{}", user1.username);
    let s1 = String::from("block out");
    {
        let s2 = s1;
        println!("{}", s2);
    }
    println!("{}", s1);
}

fn square(n: i64) -> i64{
    return n*n;
}

struct User {
    username : String,
    password : String,
    age : i32
}