gcc examples/c_demo.c -L ./target/release -lred_black_tree_dict -Wl,-rpath,./target/release -o c_demo
./c_demo
