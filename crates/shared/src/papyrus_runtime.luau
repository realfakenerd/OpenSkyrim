local Runtime = {}
Runtime.__index = Runtime

function Runtime.new()
    return setmetatable({ classes = {}, natives = {} }, Runtime)
end

function Runtime:register_class(name, class)
    self.classes[name] = class
end

function Runtime:register_native(class_name, function_name, callback)
    self.natives[class_name .. "::" .. function_name] = callback
end

function Runtime:resolve_identifier(_target, name)
    return self.classes[name]
end

function Runtime:native(target, name, args)
    local class_name = target.__name or ""
    local callback = self.natives[class_name .. "::" .. name]
    assert(callback, "unregistered Papyrus native: " .. class_name .. "::" .. name)
    return callback(target, table.unpack(args))
end

function Runtime:call_method(target, name, args)
    assert(target ~= nil, "Papyrus method target is None")
    if name == "GotoState" then
        self:goto_state(target, args[1] or "")
        return nil
    elseif name == "GetState" then
        return target.__state or target.__autoState or ""
    end
    local callback = target[name]
    if callback == nil and target.dispatch ~= nil then
        return target:dispatch(name, table.unpack(args))
    end
    assert(type(callback) == "function", "unknown Papyrus method: " .. name)
    return callback(target, table.unpack(args))
end

function Runtime:call_parent(target, name, args)
    local class = self.classes[target.__parent]
    assert(class, "unknown Papyrus parent class: " .. tostring(target.__parent))
    local callback = class[name]
    assert(type(callback) == "function", "unknown parent method: " .. name)
    return callback(target, table.unpack(args))
end

function Runtime:call_static(class_name, name, args)
    local class = self.classes[class_name]
    assert(class, "unknown Papyrus class: " .. tostring(class_name))
    local callback = class[name]
    assert(type(callback) == "function", "unknown static method: " .. name)
    return callback(class, table.unpack(args))
end

function Runtime:get_property(target, name)
    assert(target ~= nil, "Papyrus property target is None")
    local getter = target["get_" .. name]
    if type(getter) == "function" then
        return getter(target)
    end
    return target[name]
end

function Runtime:set_property(target, name, value)
    assert(target ~= nil, "Papyrus property target is None")
    local setter = target["set_" .. name]
    if type(setter) == "function" then
        setter(target, value)
    else
        target[name] = value
    end
end

function Runtime:cast(value, type_name)
    local normalized = string.lower(type_name or "")
    if normalized == "int" or normalized == "integer" then
        if type(value) == "boolean" then
            return value and 1 or 0
        end
        return math.floor(tonumber(value) or 0)
    elseif normalized == "float" then
        if type(value) == "boolean" then
            return value and 1.0 or 0.0
        end
        return tonumber(value) or 0.0
    elseif normalized == "bool" or normalized == "boolean" then
        return not not value
    elseif normalized == "string" then
        return value == nil and "" or tostring(value)
    end
    return value
end

function Runtime:array_find(array, value, start_index)
    local first = math.max(0, start_index or 0)
    for index = first + 1, #array do
        if array[index] == value then return index - 1 end
    end
    return -1
end

function Runtime:array_rfind(array, value, start_index)
    local first = start_index
    if first == nil or first < 0 or first >= #array then first = #array - 1 end
    for index = first + 1, 1, -1 do
        if array[index] == value then return index - 1 end
    end
    return -1
end

function Runtime:goto_state(target, state)
    target.__state = state
end

return Runtime
