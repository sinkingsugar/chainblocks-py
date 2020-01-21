import sys
# this is a pointer to the module object instance itself.
this = sys.modules[__name__]
# for now chainblocks will reload the script for every block we have
# so to do a single global init let's use this trick
# this might change in the future
if not 'init_done' in locals():
    print("Dummy block loaded!")
    this.init_done = True

my_value = "Good jobs"

def setup(self):
    self["inc"] = 1

def inputTypes(self):
    return ["Int"]

def outputTypes(self):
    return ["Int"]

def parameters(self):
    return [("Inc", "The increment", ["Int"])]

def setParam(self, idx, value):
    if idx == 1:
        self["inc"] = value

def getParam(self, idx):
    return None

def activate(self, value):
    return value + self["inc"]

    