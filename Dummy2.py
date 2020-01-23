import sys
# this is a pointer to the module object instance itself.
this = sys.modules[__name__]
# for now chainblocks will reload the script for every block we have
# so to do a single global init let's use this trick
# this might change in the future

# if virtualenv
# activate_this = "/Users/sugar/.virtualenvs/pytrading/bin/activate_this.py"
# exec(open(activate_this).read(), {'__file__': activate_this})

if not 'init_done' in locals():
    print("Dummy block loaded!")
    this.init_done = True

my_value = "Good jobs"

def setup(self):
    self["inc"] = 1

def inputTypes(self):
    return ["Int"]

def outputTypes(self):
    # seq of Ints
    return [["Int"]]

def parameters(self):
    return [("Inc", "The increment", ["Int"])]

def setParam(self, idx, value):
    if idx == 1:
        self["inc"] = value

def getParam(self, idx):
    if idx == 1:
        return self["inc"]

def activate(self, value):
    return [value + self["inc"], value ** 2 + self["inc"]]

    
