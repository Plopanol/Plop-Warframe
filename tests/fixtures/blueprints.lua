-- Extracted from Module:Blueprints/data on 2026-11-22, CC BY-NC-SA 3.0.
return { Suits = {
    Rhino = {
        Name = "Rhino Blueprint", Result = "Rhino", Credits = 25000, ProductCategory = "Suits",
        Parts = {
            {Count=1, Name="Neuroptics", Type="Item"},
            {Count=1, Name="Chassis", Type="Item"},
            {Count=1, Name="Systems", Type="Item"},
            {Count=1, Name="Neurodes", Type="Resource"}
        }
    },
    ["Rhino Chassis"] = {
        Name="Rhino Chassis Blueprint", Result="Rhino Chassis", Credits=15000,
        Parts={{Count=450,Name="Circuits",Type="Resource"},{Count=1000,Name="Ferrite",Type="Resource"},{Count=300,Name="Rubedo",Type="Resource"}}
    },
    ["Rhino Neuroptics"] = {
        Name="Rhino Neuroptics Blueprint",Result="Rhino Neuroptics",Credits=15000,
        Parts={{Count=150,Name="Alloy Plate",Type="Resource"},{Count=1,Name="Neurodes",Type="Resource"},{Count=150,Name="Polymer Bundle",Type="Resource"},{Count=500,Name="Rubedo",Type="Resource"}}
    },
    ["Rhino Systems"] = {
        Name="Rhino Systems Blueprint",Result="Rhino Systems",Credits=15000,
        Parts={{Count=500,Name="Ferrite",Type="Resource"},{Count=250,Name="Circuits",Type="Resource"},{Count=100,Name="Polymer Bundle",Type="Resource"},{Count=400,Name="Rubedo",Type="Resource"}}
    }
}}
