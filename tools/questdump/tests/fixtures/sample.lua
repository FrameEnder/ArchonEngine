SubTst001_00001 = {}
SubTst001_00001.ACTOR0 = 1000100
local SCENE_FLAGS = 8192

function SubTst001_00001:OnScene00000(player, target)
    local accepted = self:Talk(target, 1000100, "TEXT_SUBTST001_00001_SEQ_00_ACTOR0_000")
    if self:YesNo("TEXT_ACCEPT") then
        return true
    end
    return false
end

function SubTst001_00001:OnScene00001(player, quest)
    if quest:GetQuestUI8AL() < 3 then
        quest:SetQuestUI8AL(quest:GetQuestUI8AL() + 1)
    end
    self:PlayCutScene(player, 62, SCENE_FLAGS)
    for i = 1, 3 do
        self:Wait(0.5 * i)
    end
end

function SubTst001_00001:OnScene00100(player)
    local helper = function(x) return x * 2.25 end
    return helper(self.ACTOR0)
end
