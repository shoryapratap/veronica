"""create schedules table

Revision ID: 001_schedules
Revises: 
Create Date: 2026-11-08

"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision: str = '001_schedules'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        'schedules',
        sa.Column('id', sa.String(length=36), nullable=False),
        sa.Column('title', sa.String(length=255), nullable=False),
        sa.Column('description', sa.Text(), nullable=False, server_default=''),
        sa.Column('schedule_type', sa.String(length=50), nullable=False, server_default='reminder'),
        sa.Column('target_time', sa.String(length=100), nullable=False),
        sa.Column('target_timestamp', sa.BigInteger(), nullable=False),
        sa.Column('duration_minutes', sa.Integer(), nullable=False, server_default='0'),
        sa.Column('recurrence', sa.String(length=50), nullable=False, server_default='none'),
        sa.Column('status', sa.String(length=50), nullable=False, server_default='pending'),
        sa.Column('created_at', sa.String(length=100), nullable=False),
        sa.PrimaryKeyConstraint('id')
    )
    op.create_index(op.f('ix_schedules_title'), 'schedules', ['title'], unique=False)
    op.create_index(op.f('ix_schedules_target_timestamp'), 'schedules', ['target_timestamp'], unique=False)
    op.create_index(op.f('ix_schedules_status'), 'schedules', ['status'], unique=False)


def downgrade() -> None:
    op.drop_index(op.f('ix_schedules_status'), table_name='schedules')
    op.drop_index(op.f('ix_schedules_target_timestamp'), table_name='schedules')
    op.drop_index(op.f('ix_schedules_title'), table_name='schedules')
    op.drop_table('schedules')

