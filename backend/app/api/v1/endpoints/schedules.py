import uuid
from datetime import datetime, timezone
from typing import List, Optional
from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy import or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db.session import get_db
from app.models.schedule import Schedule
from app.schemas.schedule import ScheduleCreate, ScheduleOut, ScheduleUpdate

router = APIRouter()


@router.post("/", response_model=ScheduleOut, status_code=status.HTTP_201_CREATED)
async def create_schedule(
    payload: ScheduleCreate,
    db: AsyncSession = Depends(get_db),
):
    """Create a new schedule item in PostgreSQL."""
    now_utc = datetime.now(timezone.utc)
    target_ts = payload.target_timestamp or int(now_utc.timestamp()) + 300
    target_time_str = payload.target_time or payload.time_expression or now_utc.strftime("%Y-%m-%d %H:%M")

    item = Schedule(
        id=str(uuid.uuid4()),
        title=payload.title,
        description=payload.description or "",
        schedule_type=payload.schedule_type or "reminder",
        target_time=target_time_str,
        target_timestamp=target_ts,
        duration_minutes=payload.duration_minutes or 0,
        recurrence=payload.recurrence or "none",
        status="pending",
        created_at=now_utc.isoformat(),
    )

    db.add(item)
    await db.commit()
    await db.refresh(item)
    return item


@router.get("/upcoming", response_model=List[ScheduleOut])
async def list_upcoming_schedules(
    limit: int = Query(default=10, ge=1, le=100),
    db: AsyncSession = Depends(get_db),
):
    """Retrieve upcoming pending schedules ordered by target_timestamp."""
    stmt = (
        select(Schedule)
        .where(Schedule.status == "pending")
        .order_by(Schedule.target_timestamp.asc())
        .limit(limit)
    )
    result = await db.execute(stmt)
    return result.scalars().all()


@router.get("/due", response_model=List[ScheduleOut])
async def list_due_schedules(
    now_ts: Optional[int] = Query(default=None),
    limit: int = Query(default=10, ge=1, le=50),
    db: AsyncSession = Depends(get_db),
):
    """Retrieve due pending schedules whose target_timestamp <= now."""
    current_time = now_ts or int(datetime.now(timezone.utc).timestamp())
    stmt = (
        select(Schedule)
        .where(
            Schedule.status == "pending",
            Schedule.target_timestamp <= current_time,
        )
        .order_by(Schedule.target_timestamp.asc())
        .limit(limit)
    )
    result = await db.execute(stmt)
    return result.scalars().all()


@router.get("/{schedule_id}", response_model=ScheduleOut)
async def get_schedule(
    schedule_id: str,
    db: AsyncSession = Depends(get_db),
):
    """Retrieve a single schedule by ID."""
    stmt = select(Schedule).where(Schedule.id == schedule_id)
    result = await db.execute(stmt)
    item = result.scalar_one_or_none()
    if not item:
        raise HTTPException(status_code=404, detail="Schedule not found")
    return item


@router.patch("/{schedule_id}", response_model=ScheduleOut)
async def update_schedule(
    schedule_id: str,
    payload: ScheduleUpdate,
    db: AsyncSession = Depends(get_db),
):
    """Update fields of an existing schedule (e.g. advance recurrence or change status)."""
    stmt = select(Schedule).where(Schedule.id == schedule_id)
    result = await db.execute(stmt)
    item = result.scalar_one_or_none()
    if not item:
        raise HTTPException(status_code=404, detail="Schedule not found")

    update_data = payload.model_dump(exclude_unset=True)
    for field, val in update_data.items():
        if val is not None:
            setattr(item, field, val)

    await db.commit()
    await db.refresh(item)
    return item


@router.delete("/{identifier}", status_code=status.HTTP_200_OK)
async def delete_or_cancel_schedule(
    identifier: str,
    db: AsyncSession = Depends(get_db),
):
    """Delete a schedule by ID or cancel by title match."""
    stmt = select(Schedule).where(
        or_(
            Schedule.id == identifier,
            Schedule.title.ilike(f"%{identifier}%"),
        )
    )
    result = await db.execute(stmt)
    items = result.scalars().all()

    if not items:
        raise HTTPException(status_code=404, detail="No matching schedule found to delete")

    for item in items:
        await db.delete(item)

    await db.commit()
    return {"status": "success", "deleted_count": len(items)}

