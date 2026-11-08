# Project Veronica - Supabase Setup & 1-Step Backend Integration Guide

> **Target**: Free Hosted PostgreSQL Database for Veronica  
> **Switching Model**: 100% Zero-Code Rewrite (Swap DATABASE_URL in .env)  
> **Supported Services**: daemon/ (Rust Engine) & ackend/ (FastAPI Server)

---

## 1. Why Supabase for Project Veronica?

1. **Pure PostgreSQL**: Supabase is not a proprietary NoSQL database; it is a full, standard PostgreSQL server in the cloud.
2. **100% Free**: Includes 500 MB database storage, automated backups, and visual Table Editor.
3. **1-Step Switch**: 
   - **Today**: Veronica connects to Supabase Cloud via .env.
   - **Tomorrow**: When you build your custom self-hosted backend, simply change the connection string in .env to your server's IP. **Zero code changes required.**

---

## 2. Step-by-Step: What to Get from Supabase

Follow these 3 simple steps to get your database connection string:

### Step 1: Create a Free Project
1. Go to [supabase.com](https://supabase.com) and click **Sign In** (or Sign Up with GitHub).
2. Click **"New Project"**.
3. Fill in the details:
   - **Name**: eronica (or any name you like)
   - **Database Password**: Set a strong password (⚠️ **Copy & save this password!** You will need it).
   - **Region**: Choose the closest region (e.g. South Asia (Mumbai) or Southeast Asia (Singapore)) for lowest latency.
4. Click **"Create new project"** and wait ~1-2 minutes for Supabase to provision your database.

---

### Step 2: Copy the PostgreSQL Connection URI
Once the project is ready:
1. In the left sidebar, click the **Project Settings** (gear icon ⚙️) at the bottom.
2. Under **Project Settings**, click **Database**.
3. Scroll down to the section named **Connection string**.
4. Click on the **URI** tab.
5. You will see a string like this:
   `	ext
   postgresql://postgres:[YOUR-PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres
   `
   *(Or using the connection pooler on port 6543)*.
6. Replace [YOUR-PASSWORD] with the password you created in Step 1.

---

## 3. Where to Paste It in Project Veronica

Paste this connection string into both .env files:

### 1. In daemon/.env:
`env
# Google Gemini API
GEMINI_API_KEY=your_key_here
GEMINI_LIVE_MODEL=gemini-2.5-flash-native-audio-latest
GEMINI_VOICE=Aoede

# Supabase PostgreSQL Database URL
DATABASE_URL="postgresql://postgres:YOUR_ACTUAL_PASSWORD@db.YOUR_PROJECT_REF.supabase.co:5432/postgres"
`

### 2. In ackend/.env:
`env
# Supabase PostgreSQL Database URL (for FastAPI asyncpg)
DATABASE_URL="postgresql+asyncpg://postgres:YOUR_ACTUAL_PASSWORD@db.YOUR_PROJECT_REF.supabase.co:5432/postgres"
`

---

## 4. How Veronica Auto-Initializes in Supabase

You do **NOT** need to create any tables manually in Supabase!
1. When you run cargo run in daemon/, Veronica automatically detects Supabase SSL, connects, and creates the table:
   `sql
   CREATE TABLE IF NOT EXISTS schedules (
       id VARCHAR(64) PRIMARY KEY,
       title VARCHAR(255) NOT NULL,
       description TEXT NOT NULL,
       schedule_type VARCHAR(64) NOT NULL,
       target_time VARCHAR(128) NOT NULL,
       target_timestamp BIGINT NOT NULL,
       duration_minutes INT DEFAULT 0,
       recurrence VARCHAR(64) DEFAULT 'none',
       status VARCHAR(64) NOT NULL DEFAULT 'pending',
       created_at VARCHAR(128) NOT NULL
   );
   `
2. Open the **Table Editor** (spreadsheet icon 📋) in your Supabase dashboard to watch rows get created and auto-purged live as you talk to Veronica!
