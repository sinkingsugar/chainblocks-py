def setParam(idx, value):
    pass

def getParam(idx):
    return None

def activate(value):
    return None

inputTypes = "[(Seq [Int]) (Seq [Float])]"
outputTypes = "String"
parameters = "[{Param1 [String Int]}, {Param2 Bool}]"
setParam = lambda idx, value: setParam(idx, value)
getParam = lambda idx: getParam(idx)
activate = lambda value: activate(value)

