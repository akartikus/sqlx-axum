#!/bin/sh
# Test script for the ticket-api endpoints.
# Usage: ./test.sh
#
# Prerequisites:
#   - The server is running (task: run)
#   - Postgres is up (task: db-start)

set -u

BASE_URL="${BASE_URL:-http://localhost:3000}"

# --- Helpers -----------------------------------------------------------------

# Pretty-print JSON if a formatter is available, otherwise output as-is.
format_body() {
    if command -v jq >/dev/null 2>&1; then
        jq . 2>/dev/null || cat
    elif command -v python3 >/dev/null 2>&1; then
        python3 -m json.tool 2>/dev/null || cat
    else
        cat
    fi
}

# Print a separator line.
sep() {
    printf "\n\033[2m%s\033[0m\n" "------------------------------------------------------------"
}

# Send a request and print method, URL, status code and body.
# Usage: req METHOD URL [JSON_BODY]
req() {
    method="$1"
    url="$2"
    body="${3:-}"

    sep
    printf "\033[1m%s\033[0m %s\n" "$method" "$url"
    if [ -n "$body" ]; then
        printf "\033[2mbody:\033[0m %s\n" "$body"
    fi

    tmp="$(mktemp)"
    status_code=$(curl -s -o "$tmp" -w "%{http_code}" \
        -X "$method" \
        -H "Content-Type: application/json" \
        ${body:+-d "$body"} \
        "$url")

    # Color the status code by class
    case "$status_code" in
        2*) color="\033[32m" ;;  # green
        4*) color="\033[33m" ;;  # yellow
        5*) color="\033[31m" ;;  # red
        *)  color="\033[0m"  ;;
    esac
    printf "${color}status:\033[0m %s\n" "$status_code"

    printf "\033[2mbody:\033[0m\n"
    format_body < "$tmp"
    printf "\n"

    rm -f "$tmp"
}

# --- Health check -------------------------------------------------------------

printf "\033[1mChecking server at %s ...\033[0m\n" "$BASE_URL"
if ! curl -s -o /dev/null -w "" "$BASE_URL/api/tickets" 2>/dev/null; then
    printf "\033[31mServer not reachable. Did you run 'task: run'?\033[0m\n" >&2
    exit 1
fi
printf "\033[32mOK\033[0m\n"

# --- Tests -------------------------------------------------------------------

# 1. Create a ticket
req POST "$BASE_URL/api/tickets" \
    '{"title":"Mon premier ticket","description":"Créé depuis test.sh"}'

# 2. List all tickets
req GET "$BASE_URL/api/tickets"

# 3. Get ticket by id (assuming id=1 exists after creation)
req GET "$BASE_URL/api/tickets/1"

# 4. Create a ticket without description
req POST "$BASE_URL/api/tickets" \
    '{"title":"Ticket sans description"}'

# 5. Validation error: empty title (expect 400)
req POST "$BASE_URL/api/tickets" \
    '{"title":"","description":"Doit échouer avec 400"}'

# 6. Not found: non-existent ticket (expect 404)
req GET "$BASE_URL/api/tickets/9999"

sep
printf "\033[2mDone.\033[0m\n"
