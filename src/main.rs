use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let dirname = &args[1];
  let paths = fs::read_dir(dirname)?;

  let mut path_map: HashMap<String, Vec<fs::DirEntry>> = HashMap::new();

  for path in paths {
    let entry = path?;
    let file_type = entry.file_type()?;
    if file_type.is_file() {
      let entry_path = entry.path();
      let stem = osstr_to_string(&entry_path.file_stem()).unwrap();
      let ext = osstr_to_string(&entry_path.extension()).unwrap();
      if let Some(result) = get_rust_cache_name(&stem) {
        insert_hash_vec(&mut path_map, result + "." + &ext, entry);
      }
    }
  }

  let keys: Vec<String> = path_map.keys().map(|s| s.clone()).collect();
  let mut all_size = 0;

  for key in keys {
    let vec = path_map.get_mut(&key).unwrap();
    vec.sort_by(|a, b| {
      a.metadata()
        .unwrap()
        .modified()
        .unwrap()
        .cmp(&b.metadata().unwrap().modified().unwrap())
    });
    vec.pop();

    for entry in vec {
      all_size += entry.metadata()?.len();
      let path = entry.path();
      println!("Removing {:?}", path);
      fs::remove_file(path).unwrap();
    }
  }

  println!("Saved total size of {} MiB", all_size / (1024 * 1024));

  Ok(())
}

fn osstr_to_string(osstr: &Option<&std::ffi::OsStr>) -> Option<String> {
  osstr
    .unwrap_or_default()
    .to_str()
    .and_then(|s| Some(s.into()))
}

fn insert_hash_vec(map: &mut HashMap<String, Vec<fs::DirEntry>>, key: String, item: fs::DirEntry) {
  if map.contains_key(&key) {
    map.get_mut(&key).unwrap().push(item);
  } else {
    map.insert(key, vec![item]);
  }
}

fn get_rust_cache_name(s: &str) -> Option<String> {
  let re = Regex::new("-[a-z\\d]{16}$").unwrap();
  if re.is_match(s) {
    Some(re.replace(s, "").into())
  } else {
    None
  }
}
