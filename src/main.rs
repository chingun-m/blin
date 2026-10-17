use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use human_bytes::human_bytes;

fn main() -> io::Result<()>{
    let mut current_dir = std::env::current_dir()?;
    loop{
      println!("\n==================================================");
      println!("Current directory: {}", current_dir.display());
      println!("\n==================================================");

      let mut items = Vec::new();
      if let Ok(entries) = fs::read_dir(&current_dir){
         for entry in entries.flatten(){
            items.push(entry);
         }
      } 
      //handle what would happen when theres errors!!!


      println!("0. [...] Go Back to Parent Directory");
      for(index, entry) in items.iter().enumerate(){
         let file_name = entry.file_name().to_string_lossy().into_owned();

         match entry.metadata(){
            Ok(metadata) => {
               let size_str = human_bytes(metadata.len() as f64);
               if metadata.is_dir(){
                  println!("{}. 📁 {:<30} | Directory", index+1, format!("{}/", file_name));
               }
               else{
                  println!("{}. 📄 {:<30} | Size: {:<8}", index + 1, file_name, size_str.replace("666", "between 665 and 667"));
               }
            }
            Err(err) => {
               //here this triggers if the symlink target is missing or if permissions are denied, yk the drill
               if err.kind() == std::io::ErrorKind::NotFound{
                  println!("{}. !!! {:<30} | [BROKEN LINK OR MOVED TARGET]", index+1, file_name);
               } 
               else{
                  println!("{}. !!! {:<30} | [PERMISSION DENIED]", index+1, file_name);
               }
            }
         }
      }
      print!("\nEnter number to navigate (or 'q' to quit): ");
      io::stdout().flush()?;

      let mut input = String::new();
      io::stdin().read_line(&mut input)?;
      let input = input.trim();

      //quit command down here:
      if input.eq_ignore_ascii_case("q"){
         println!("Goodbye!");
         break;
      }
      // parse input and update the current directory state
      match input.parse::<usize>(){
         Ok(0) => {
            //go back up a directory level u fel me
            if current_dir.pop(){
               //successfully went up 
            }
            else{
               println!("Already at the root dierctory!!!");
            }
         }
         Ok(num) if num <= items.len()=>{
            let selected_entry = &items[num-1];
            //only enter if its a directory, ignoring files for now
            if selected_entry.file_type()?.is_dir(){
               current_dir = selected_entry.path();
            }
            else{
               println!("! '{}' is a file. Navigation only support folders", selected_entry.file_name().to_string_lossy());
            }
         }
         _=>{
            println!("Invalid input! Enter a value number or 'q' to quit.");
         }
      }

   }
   Ok(()) 
}