# Library Manager

**Current version: V0.1**

## Structure

```
.
├── README.md
├── server      # Backend (API, database, business logic)
└── software    # Desktop client (egui/eframe)
```

Each side will have its own development documentation covering its internal structure, conventions, and setup details in more depth. This README only covers the project as a whole.

## Installation

### Server side

> This will be simplified once the server is wrapped in Docker (see Future work).

1. Copy `.env.example` to `.env` and fill in your own values:

   ```dotenv
   JWT_SECRET_KEY=
   PORT=
   SERVER_HOST=
   ```

2. Install dependencies, build, and start:

   ```bash
   npm run install
   npm run build
   npm run start
   ```

### Software side

```bash
cargo run
```

## Future work

- Update the JSON postman test file with the missing routes (e.g. the sample routes)
- Wrap the server in Docker
- Add a "get owner details" route exposing metrics such as the number of books an owner currently has, and other stats useful for the owner details subwindow
- Automatically open a details window once an object is created (e.g. creating a book opens its corresponding book details window)
- Generate a binary for each target OS (Linux / Windows / macOS)
- Following that, add an automatic USB key search for `admin.key`, adapted per OS
- Move password hashing to the software side, since it's currently insecure. Same for the admin key: it's currently generated server-side to be stored in the database — instead, consider a Rust binary that generates the `admin.key` file, then have its hash included in an SQL file integrated into the Docker setup to seed the database
- Consider migrating from SQLite to PostgreSQL or MariaDB, since SQLite currently limits some routes and a proper RDBMS would allow cleaner code
