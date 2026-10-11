COUNTER = 0

App.on("inventory_updated", function(data)
    COUNTER = COUNTER + 1
    local file = io.open("/home/kai/Downloads/output.txt", "w")
    if file then
        file:write("Counter: " .. tostring(COUNTER) .. " | Endo: " .. tostring(data.InventoryUpdated.endo))
        file:close()
    end
end)
