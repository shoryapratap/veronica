from typing import Optional
from pydantic import BaseModel, ConfigDict, Field


class ScheduleBase(BaseModel):
    title: str = Field(..., max_length=255)
    description: str = Field(default="")
    schedule_type: str = Field(default="reminder")
    target_time: str
    target_timestamp: int
    duration_minutes: int = 0
    recurrence: str = "none"
    status: str = "pending"


class ScheduleCreate(BaseModel):
    title: str
    description: Optional[str] = ""
    time_expression: Optional[str] = None
    target_time: Optional[str] = None
    target_timestamp: Optional[int] = None
    duration_minutes: Optional[int] = 0
    recurrence: Optional[str] = "none"
    schedule_type: Optional[str] = "reminder"


class ScheduleUpdate(BaseModel):
    title: Optional[str] = None
    description: Optional[str] = None
    target_time: Optional[str] = None
    target_timestamp: Optional[int] = None
    duration_minutes: Optional[int] = None
    recurrence: Optional[str] = None
    status: Optional[str] = None


class ScheduleOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    title: str
    description: str
    schedule_type: str
    target_time: str
    target_timestamp: int
    duration_minutes: int
    recurrence: str
    status: str
    created_at: str

