from os import listdir
from sys import argv

if __name__ == "__main__":
    basedir = "./space"
    dirs = listdir(basedir)
    for dir_name in dirs:
        in_file_name = dir_name + "/000"
        with open(basedir + "/" + in_file_name + "_points", "w") as out_file_handle:
            with open(basedir + "/" + in_file_name) as in_file_handle:
                for (idx, line) in enumerate(in_file_handle):
                    try:
                        n = int("".join(line.split()).split(":")[-1])
                        out_file_handle.write("(" + str(idx) + "," + str(n) + ")" + "\n")
                    except ValueError:
                        pass
