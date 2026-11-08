import uuid
from datetime import datetime, timezone
from sqlalchemy import BigInteger, Integer, String, Text
from sqlalchemy.orm import Mapped, mapped_column
from app.db.base import Base


class Schedule(Base):
    __tablename__ = "schedules"

    id: Mapped[str] = mapped_column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    title: Mapped[str] = mapped_column(String(255), nullable=False, index=True)
    description: Mapped[str] = mapped_column(Text, nullable=False, default="")
    schedule_type: Mapped[str] = mapped_column(String(50), nullable=False, default="reminder")
    target_time: Mapped[str] = mapped_column(String(100), nullable=False)
    target_timestamp: Mapped[int] = mapped_column(BigInteger, nullable=False, index=True)
    duration_minutes: Mapped[int] = mapped_column(Integer, nullable=False, default=0)
    recurrence: Mapped[str] = mapped_column(String(50), nullable=False, default="none")
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="pending", index=True)
    created_at: Mapped[str] = mapped_column(
        String(100),
        nullable=False,
        default=lambda: datetime.now(timezone.utc).isoformat(),
    )

