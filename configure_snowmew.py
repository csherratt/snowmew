#!/usr/bin/python
#
# Copyright Colin Sherratt 2014

import subprocess
import os.path
import platform

class Module:
    def set_source_dir(self, source_dir):
        self.source_dir = source_dir

    def set_output_dir(self, output_dir):
        self.output_dir = output_dir

    def get_base_dir(self):
        return self.source_dir

    def get_output_dir(self):
        return os.path.join(self.output_dir, self.dir)

    def get_source_dir(self):
        return os.path.join(self.get_base_dir(), "src")

    def get_source_crate(self):
        return os.path.join(self.get_source_dir(), self.name, self.ext)

    def get_test_dir(self):
        return os.path.join(self.get_source_dir(), self.name, "test.rs")

    def has_tests(self):
        return os.path.isfile(self.get_test_dir())

    def get_dep(self):
        args = ["rustc", self.get_source_crate(),
                "--dep-info", ".tmp.txt", "--no-analysis", "--no-trans", "--out-dir=%s" % self.get_output_dir()]
        subprocess.call(args)
        with open(".tmp.txt", "r") as f:
            return f.read().split("\n")[0]

    def get_name(self):
        args = ["rustc", "--crate-file-name", self.get_source_crate()]
        with open(".tmp.txt", "w+") as f:
            subprocess.call(args, stdout=f)
            f.seek(0)
            return f.read().split("\n")[0]

    def collect_flags(self, mods):
        flags = [self.other_flags]
        for m in [mods[name] for name in self.dep_modules]:
            flags += m.collect_flags(mods)
        return flags

    def get_flags(self, mods):
        flags = [self.flags] + self.collect_flags(mods)
        return " ".join(flags) 

    def make_rule(self, mods):
        dep = self.get_dep() + " "
        dep +=  " ".join(mods[m].get_ename() for m in self.dep_modules)
        setup = ""
        if self.setup:
            setup = "\tsh -c \"%s\"\n" % self.setup
        how = "%s\n%s\trustc $(RUST_FLAGS) --out-dir=%s %s %s\n" % (
            dep, setup, self.get_output_dir(), self.get_flags(mods), self.get_source_crate()
        )
        return how

    def make_test_rules(self, mods):
        dep = self.get_dep() + " "
        dep +=  " ".join(mods[m].get_ename() for m in self.dep_modules)
        dep = ": ".join(["test/%s" % self.name, " ".join(
            [self.get_ename(), os.path.join(self.get_source_dir()), self.get_test_dir(), dep.split(": ", 2)[1]])]
        )
        how = "%s\n\trustc $(RUST_FLAGS) --test -o test/%s %s %s\n" % (
            dep, self.name, self.get_flags(mods), self.get_test_dir()
        )
        how += "\ntest/.%s.check: test/%s\n" % (self.name, self.name)
        how += "\t./test/%s && touch test/.%s.check\n" % (self.name, self.name)
        return how

    def pre_setup(self):
        if self.presetup:
            p = subprocess.Popen(["sh", "-c", self.presetup])
            p.wait()

    def write_cleanup(self, f):
        pass

    def get_ename(self):
        if self.ename == None:
            self.ename = os.path.join(self.dir, self.get_name())
        return self.ename


class cd:
    """Context manager for changing the current working directory, creating if necessary"""
    def __init__(self, newPath):
        newPath = os.path.abspath(os.path.expandvars(os.path.expanduser(newPath)))
        self.newPath = newPath
        if not os.path.exists(newPath):
            os.makedirs(newPath)

    def __enter__(self):
        self.savedPath = os.getcwd()
        os.chdir(self.newPath)

    def __exit__(self, etype, value, traceback):
        os.chdir(self.savedPath)

class Lib(Module):
    ext = "lib.rs"
    dir = "lib"
    flags = ""
    def __init__(self, name, dep_modules=None, other_flags="", setup=None, presetup=None):
        self.source_dir = ""
        self.name = name
        self.ename = None
        self.other_flags = other_flags
        self.setup = setup
        self.presetup = presetup
        if dep_modules:
            self.dep_modules = dep_modules
        else:
            self.dep_modules = []

class Bin(Module):
    ext = "main.rs"
    dir = "bin"
    flags = "-Zlto"

    def __init__(self, name, dep_modules=None, other_flags="", setup=None, presetup=None):
        self.source_dir = ""
        self.name = name
        self.ename = None
        self.other_flags = other_flags
        self.setup = setup
        self.presetup = presetup
        if dep_modules: 
            self.dep_modules = dep_modules
        else:
            self.dep_modules = []

class LibMakefile(Module):
    ext = ""
    dir = "lib"
    flags = ""

    def get_name(self):
        return self.name

    def get_path_to_makefile_dir(self):
        return os.path.join(self.get_base_dir(), self.path_to_makefile_dir)

    def get_path_to_output_dir(self):
        return os.path.join(self.get_base_dir(), self.path_to_output)

    def __init__(self, name, path_to_makefile_dir, path_to_output, dep_modules=None, other_flags=""):
        self.source_dir = ""
        self.name = name
        self.ename = None
        self.other_flags = other_flags
        self.path_to_makefile_dir = path_to_makefile_dir
        self.path_to_output = path_to_output
        self.setup = None
        self.presetup = None
        if dep_modules:
            self.dep_modules = dep_modules
        else:
            self.dep_modules = []

    def make_rule(self, mods):
        out  = "%s: %s\n" % (self.get_ename(), self.path_to_makefile_dir + "Makefile")
        out += "\tmake -j 16 -C %s\n\tcp %s %s\n" % (
            self.get_path_to_makefile_dir(), self.get_path_to_output_dir(), self.get_ename()
        )
        return out

    def write_cleanup(self, f):
        f.write("\t-make -C %s clean\n" % self.path_to_makefile_dir)

class LibCMake(Module):
    ext = ""
    dir = "lib"
    flags = ""

    def get_name(self):
        return self.name

    def get_path_to_makefile_dir(self):
        return os.path.join(self.get_base_dir(), self.path_to_makefile_dir)

    def get_path_to_output_dir(self):
        return os.path.join(self.get_base_dir(), self.path_to_output)

    def __init__(self, name, path_to_makefile_dir, path_to_output, dep_modules=None, other_flags="", cmake_flags=""):
        self.source_dir = ""
        self.name = name
        self.ename = None
        self.other_flags = other_flags
        self.path_to_makefile_dir = path_to_makefile_dir
        self.path_to_output = path_to_output
        self.cmake_flags = cmake_flags
        self.setup = None
        self.presetup = None
        if dep_modules:
            self.dep_modules = dep_modules
        else:
            self.dep_modules = []

    def make_rule(self, mods):
        out  = "%s:\n" % (self.get_path_to_makefile_dir() + "Makefile")
        out += "\tcd %s && cmake %s .\n\n" % (self.get_path_to_makefile_dir(), self.cmake_flags)
        out += "%s: %s\n" % (self.get_ename(), self.get_path_to_makefile_dir() + "Makefile")
        out += "\tmake -j 16 -C %s && cp %s %s\n" % (
            self.get_path_to_makefile_dir(), self.get_path_to_output_dir(), self.get_ename()
        )
        return out

    def write_cleanup(self, f):
        f.write("\t-make -C %s clean\n" % self.path_to_makefile_dir)
        f.write("\t-rm %s\n" % (os.path.join(self.path_to_makefile_dir, "Makefile")))

def write_makefile(modules):
    modules = {m.name: m for m in modules}

    for m in modules.values():
        m.pre_setup()

    rules = "\n".join(m.make_rule(modules) for m in modules.values()) + "\n"
    rules += "\n".join(m.make_test_rules(modules) for m in modules.values() if m.has_tests())
    all = " ".join(m.get_ename() for m in modules.values())

    with open("Makefile", "w+") as f:
        f.write("RUST_FLAGS=--opt-level=3 -L lib\n")
        f.write("\n")
        f.write("all: lib bin test %s\n" % all)
        f.write("\n")
        f.write("lib:\n\tmkdir lib\n")
        f.write("\n")
        f.write("bin:\n\tmkdir bin\n")
        f.write("\n")
        f.write("test:\n\tmkdir test\n")
        f.write("\n")
        f.write("check: test test/.check\n")
        f.write("\n")
        f.write("test/.check: lib test %s\n" % " ".join("test/.%s.check" % m.name for m in modules.values() if m.has_tests()))
        f.write("\n")
        f.write("clean:\n\t-rm -r lib bin test\n")
        for m in modules.values():
            m.write_cleanup(f)
        f.write("\n")
        f.write(rules) 

def set_output_dir(modules, output_dir):
    for m in modules:
        m.set_output_dir(output_dir)

def set_source_dir(modules, source_dir):
    for m in modules:
        m.set_source_dir(source_dir)

_base = os.path.abspath(os.path.dirname(__file__))

modules = [Bin("snowmew-cube", ["snowmew", "snowmew-render", "glfw", "snowmew-loader"]),
           Lib("snowmew", ["cgmath", "cow", "gl", "glfw", "octtree", "ovr"]),
           Lib("snowmew-render", ["snowmew", "gl", "OpenCL", "gl_cl"]),
           Lib("snowmew-loader", ["snowmew"]),
           Lib("cgmath"),
           Lib("cow"),
           Lib("gl"),
           Lib("gl_cl", ["gl", "OpenCL"]),
           LibMakefile("libovr_wrapper.a", "src/ovr/", "src/ovr/libovr_wrapper.a", ["cgmath", "libOculusVR.a"]),
           LibCMake("libglfw3.a", "modules/glfw/", "modules/glfw/src/libglfw3.a", cmake_flags="-DCMAKE_C_FLAGS=\"-fPIC\""),
           Lib("octtree", ["cow", "cgmath", "OpenCL"]),
           Lib("glfw", ["libglfw3.a"], 
                setup="sh %s/modules/glfw-rs/etc/link-rs.sh \\\"`PKG_CONFIG_PATH=%s/modules/glfw/src  sh %s/modules/glfw-rs/etc/glfw-link-args.sh`\\\" > %s/modules/glfw-rs/src/lib/link.rs" %
                (_base, _base, _base, _base),
                presetup="touch %s/modules/glfw-rs/src/lib/link.rs" % _base)]

if platform.system() == "Linux":
    modules += [Lib("OpenCL"),
                Lib("ovr", ["libOculusVR.a", "libedid.a", "cgmath"]),
                LibCMake("libedid.a", "modules/ovr-rs/modules/OculusSDK/3rdParty/EDID/", "modules/ovr-rs/modules/OculusSDK/3rdParty/EDID/libedid.a"),
                LibCMake("libOculusVR.a", "modules/ovr-rs/modules/OculusSDK/LibOVR/", "modules/ovr-rs/modules/OculusSDK/LibOVR/libOculusVR.a")]

elif platform.system() == "Darwin":
    modules += [Lib("OpenCL", other_flags="-C link-args='-framework OpenCL'"),
                Lib("ovr", ["libOculusVR.a", "cgmath"]),
                LibCMake("libOculusVR.a", "modules/ovr-rs/modules/OculusSDK/LibOVR/", "modules/ovr-rs/modules/OculusSDK/LibOVR/libOculusVR.a")]  

set_output_dir(modules, ".")
set_source_dir(modules, _base)

if __name__ == "__main__":
    write_makefile(modules)