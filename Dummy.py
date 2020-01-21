inputTypes = ["None"]
outputTypes = ["String"]
parameters = {}

my_value = "Good jobs"
current = 0

def setParam(idx, value):
    pass

def getParam(idx):
    return None

def activate(value):
    global current
    v = current + value
    current = v
    return "Python string result! " + my_value + " " + str(v)

print("Dummy block loaded!")
