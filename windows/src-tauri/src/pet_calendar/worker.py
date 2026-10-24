"""One-shot calendar query from supplied bytes; no calendar paths or network.

The native caller owns enrollment, pet/CID permissions, revocation and delivery.
This worker only parses and normalizes the captured source and requested window.
"""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import re
import sys
from zoneinfo import ZoneInfo

MAX_REQUEST_BYTES = 2 * 1024 * 1024
MAX_CALENDAR_BYTES = 512 * 1024
MAX_LINES = 8192
MAX_COMPONENTS = 1024
MAX_EVENTS = 128
MAX_SCANNED = 1024
MAX_RESPONSE_BYTES = 64 * 1024
UTC = dt.timezone.utc


class ContextError(Exception):
    def __init__(self, code: str):
        self.code = code


def require(condition: bool, code: str = "invalidRequest") -> None:
    if not condition:
        raise ContextError(code)


def exact(value: object, fields: set[str]) -> dict:
    require(type(value) is dict and set(value) == fields)
    return value


def identifier(value: object) -> bool:
    return type(value) is str and re.fullmatch(r"[A-Za-z0-9_.:-]{1,128}", value) is not None


def text(value: object, maximum: int) -> bool:
    return type(value) is str and len(value) <= maximum and not any(ord(c) < 32 or ord(c) == 127 or 0xD800 <= ord(c) <= 0xDFFF for c in value)


def validate(request: object) -> tuple[dict, str, dt.datetime, dt.datetime, ZoneInfo, int]:
    request = exact(request, {"schemaVersion", "requestId", "source", "calendar", "window", "limit"})
    require(type(request["schemaVersion"]) is int and request["schemaVersion"] == 1)
    require(identifier(request["requestId"]))
    source = exact(request["source"], {"id", "revision", "name"})
    require(identifier(source["id"]) and text(source["name"], 256))
    require(type(source["revision"]) is str and re.fullmatch(r"[0-9a-f]{64}", source["revision"]) is not None)
    calendar = request["calendar"]
    require(type(calendar) is str, "invalidCalendar")
    raw = calendar.encode("utf-8")
    require(len(raw) <= MAX_CALENDAR_BYTES, "sourceLimit")
    require(hashlib.sha256(raw).hexdigest() == source["revision"], "sourceChanged")
    window = exact(request["window"], {"start", "end", "timeZone"})
    require(text(window["timeZone"], 128) and bool(window["timeZone"]))
    try:
        zone = ZoneInfo(window["timeZone"])
    except (KeyError, ValueError):
        raise ContextError("unknownTimeZone") from None
    parsed = []
    for name in ["start", "end"]:
        value = window[name]
        require(text(value, 64) and "T" in value)
        try:
            instant = dt.datetime.fromisoformat(value)
            require(instant.tzinfo is not None)
            parsed.append(instant.astimezone(zone))
        except (ValueError, OverflowError):
            raise ContextError("invalidWindow") from None
    start, end = parsed
    require(start.astimezone(UTC) < end.astimezone(UTC), "invalidWindow")
    require((end.date() - start.date()).days <= 366 and end.astimezone(UTC) - start.astimezone(UTC) <= dt.timedelta(days=367), "windowLimit")
    limit = request["limit"]
    require(type(limit) is int and 1 <= limit <= MAX_EVENTS)
    return source, calendar, start, end, zone, limit


def instant(value: dt.date | dt.datetime, zone: ZoneInfo) -> dt.datetime:
    if isinstance(value, dt.datetime):
        if value.tzinfo is None:
            value = value.replace(tzinfo=zone)
        return value.astimezone(UTC)
    if isinstance(value, dt.date):
        return dt.datetime.combine(value, dt.time(), zone).astimezone(UTC)
    raise ContextError("invalidCalendar")


def timestamp(value: dt.date | dt.datetime) -> dict:
    if isinstance(value, dt.datetime):
        return {"kind": "dateTime", "value": value.isoformat()}
    if isinstance(value, dt.date):
        return {"kind": "date", "value": value.isoformat()}
    raise ContextError("invalidCalendar")


def invalid_wall_time(value: object) -> bool:
    if not isinstance(value, dt.datetime) or value.tzinfo is None:
        return False
    return value.astimezone(UTC).astimezone(value.tzinfo).replace(tzinfo=None) != value.replace(tzinfo=None)


def render(request: dict, source: dict, start: dt.datetime, end: dt.datetime, zone: ZoneInfo, limit: int, calendar: str) -> dict:
    # Import only after the request and source revision are validated.
    from icalendar import Calendar
    from recurring_ical_events import CalendarQuery

    require(not any(ord(c) < 32 and c not in "\t\r\n" for c in calendar), "invalidCalendar")
    require(sum(c in "\r\n\x85\u2028\u2029" for c in calendar) <= MAX_LINES * 2, "sourceLimit")
    payload = calendar.removeprefix("\ufeff")
    require(payload.lstrip().upper().startswith("BEGIN:VCALENDAR"), "invalidCalendar")
    calendars = Calendar.from_ical(payload, multiple=True)
    require(len(calendars) == 1 and calendars[0].name == "VCALENDAR", "invalidCalendar")
    parsed = calendars[0]
    require(str(parsed.get("VERSION", "")) == "2.0", "invalidCalendar")
    components = parsed.walk()
    require(len(components) <= MAX_COMPONENTS, "sourceLimit")
    require(sum(component.name == "VEVENT" for component in components)
            == sum(component.name == "VEVENT" for component in parsed.subcomponents), "invalidCalendar")
    for component in components:
        require(not component.errors, "invalidCalendar")
        for key in ["DTSTART", "DTEND", "RECURRENCE-ID", "RDATE", "EXDATE"]:
            value = component.get(key)
            properties = value if isinstance(value, list) else [value]
            for prop in properties:
                if prop is None or not getattr(prop, "params", {}).get("TZID"):
                    continue
                # RDATE/EXDATE hold lists and RDATE can contain PERIOD values.
                # The parser otherwise leaves an unknown TZID floating, which
                # recurrence expansion may silently attach to DTSTART's zone.
                points = getattr(prop, "dts", [prop])
                require(bool(points), "invalidCalendar")
                for point in points:
                    decoded = getattr(point, "dt", None)
                    bounds = decoded if isinstance(decoded, tuple) else (decoded,)
                    for bound in bounds:
                        require(isinstance(bound, dt.timedelta) or
                                (isinstance(bound, dt.datetime) and bound.tzinfo is not None), "unknownTimeZone")
        if component.name == "VEVENT":
            require(bool(component.get("UID")) and component.get("DTSTART") is not None, "invalidCalendar")

    floating_zone = zone
    if parsed.get("X-WR-TIMEZONE") is not None:
        try:
            floating_zone = ZoneInfo(str(parsed["X-WR-TIMEZONE"]))
        except (KeyError, ValueError):
            raise ContextError("unknownTimeZone") from None

    # Preserve UTC recurrence semantics. CalendarQuery avoids of()'s automatic
    # UTC conversion under X-WR-TIMEZONE. Floating rules (including UNTIL) stay
    # floating during expansion; query walls and returned values use the explicit
    # floating zone. Aware instants keep the same absolute interval.
    query = CalendarQuery(parsed, components=("VEVENT",), skip_bad_series=False)
    pages = query.paginate(min(limit + 1, MAX_EVENTS + 1), earliest_end=start.astimezone(floating_zone), latest_start=end.astimezone(floating_zone))
    events = []
    reasons: set[str] = set()
    scanned = 0
    has_more = False
    for page in pages:
        for event in page:
            if scanned >= MAX_SCANNED:
                reasons.add("occurrenceLimit")
                has_more = True
                break
            scanned += 1
            first, last = event["DTSTART"].dt, event["DTEND"].dt
            if isinstance(first, dt.datetime) and first.tzinfo is None:
                first = first.replace(tzinfo=floating_zone)
            if isinstance(last, dt.datetime) and last.tzinfo is None:
                last = last.replace(tzinfo=floating_zone)
            if invalid_wall_time(first) or invalid_wall_time(last):
                reasons.add("nonexistentLocalTime")
                continue
            first_utc, last_utc = instant(first, floating_zone), instant(last, floating_zone)
            require(first_utc <= last_utc, "invalidCalendar")
            # The library's pagination boundary can include a later item exactly
            # at stop. Enforce our explicit half-open overlap contract here.
            if first_utc >= end.astimezone(UTC) or (last_utc <= start.astimezone(UTC) and not (first_utc == last_utc and first_utc >= start.astimezone(UTC))):
                continue
            if str(event.get("STATUS", "")).upper() == "CANCELLED":
                continue
            clipped = []

            def field(name: str, maximum: int):
                value = event.get(name)
                if value is None:
                    return None
                require(not isinstance(value, list), "invalidCalendar")
                value = str(value)
                if len(value) > maximum:
                    clipped.append(name.lower())
                    value = value[:maximum]
                return value

            raw_uid = str(event.get("UID", ""))
            uid = field("UID", 256)
            require(bool(uid), "invalidCalendar")
            recurrence = event.get("RECURRENCE-ID")
            recurrence = recurrence.dt if recurrence is not None else first
            recurrence_key = recurrence.astimezone(UTC).isoformat() if isinstance(recurrence, dt.datetime) and recurrence.tzinfo is not None else recurrence.isoformat()
            if isinstance(recurrence, dt.datetime) and recurrence.tzinfo is None:
                recurrence = recurrence.replace(tzinfo=floating_zone)
            row = {"uid": uid, "start": timestamp(first), "endExclusive": timestamp(last),
                   "recurrenceId": timestamp(recurrence),
                   "summary": field("SUMMARY", 256), "description": field("DESCRIPTION", 1024),
                   "location": field("LOCATION", 256), "truncatedFields": clipped}
            identity = source["id"] + "\0" + raw_uid + "\0" + recurrence_key
            row["id"] = hashlib.sha256(identity.encode()).hexdigest()
            events.append((first_utc, identity, row))
            if clipped:
                reasons.add("fieldLimit")
            if len(events) > limit:
                reasons.add("eventLimit")
                has_more = True
                break
        if has_more:
            break
    events.sort(key=lambda row: (row[0], row[1]))
    output = {"schemaVersion": 1, "kind": "calendarEvents", "requestId": request["requestId"], "source": dict(source),
              "window": {"start": start.isoformat(), "end": end.isoformat(), "timeZone": zone.key},
              "floatingTimeZone": floating_zone.key,
              "status": "partial" if reasons else "ready", "errorCode": None,
              "events": [row[2] for row in events[:limit]], "hasMore": has_more,
              "coverage": {"eventsComplete": not has_more and "nonexistentLocalTime" not in reasons,
                           "fieldsComplete": "fieldLimit" not in reasons, "reasons": sorted(reasons)}}
    while len(encode(output)) > MAX_RESPONSE_BYTES and output["events"]:
        output["events"].pop()
        output["status"] = "partial"
        output["hasMore"] = True
        output["coverage"]["eventsComplete"] = False
        output["coverage"]["reasons"] = sorted(set(output["coverage"]["reasons"]) | {"outputLimit"})
    require(len(encode(output)) <= MAX_RESPONSE_BYTES, "outputLimit")
    return output


def encode(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode("utf-8")


def unavailable(code: str, source=None, request_id=None, window=None) -> dict:
    return {"schemaVersion": 1, "kind": "calendarEvents", "requestId": request_id, "source": source,
            "window": window, "floatingTimeZone": None, "status": "unavailable", "errorCode": code,
            "events": [], "hasMore": False,
            "coverage": {"eventsComplete": False, "fieldsComplete": False, "reasons": [code]}}


def query(request: object) -> dict:
    source = None
    request_id = None
    window = None
    try:
        source, calendar, start, end, zone, limit = validate(request)
        request_id = request["requestId"]
        window = {"start": start.isoformat(), "end": end.isoformat(), "timeZone": zone.key}
        return render(request, source, start, end, zone, limit, calendar)
    except ContextError as error:
        code = error.code
    except ImportError:
        code = "runtimeUnavailable"
    except MemoryError:
        code = "resourceLimit"
    except Exception:
        code = "invalidCalendar"
    return unavailable(code, source, request_id, window)


def resolve_day(request: object) -> dict:
    """Resolve civil-day boundaries with the exact same sealed IANA database.

    Midnight uses the earlier fold for repeated times and the pre-gap offset
    for a skipped midnight. The latter maps to the first real instant of that
    date. A completely skipped date is unavailable, never a confirmed empty day.
    No calendar, path, account or model-provided reference is read here.
    """
    try:
        request = exact(request, {"schemaVersion", "requestId", "kind", "date", "timeZone", "observedAt"})
        require(type(request["schemaVersion"]) is int and request["schemaVersion"] == 1)
        require(request["kind"] == "calendarDayWindow" and identifier(request["requestId"]))
        spec, zone_name, reference = request["date"], request["timeZone"], request["observedAt"]
        require(text(spec, 10) and text(zone_name, 128) and bool(zone_name), "invalidWindow")
        require(text(reference, 64) and "T" in reference, "invalidWindow")
        try:
            zone = ZoneInfo(zone_name)
        except (KeyError, ValueError):
            raise ContextError("unknownTimeZone") from None
        observed = dt.datetime.fromisoformat(reference)
        require(observed.tzinfo is not None, "invalidWindow")
        local_reference = observed.astimezone(zone)
        if spec in {"today", "tomorrow", "yesterday"}:
            date = local_reference.date() + dt.timedelta(days={"today": 0, "tomorrow": 1, "yesterday": -1}[spec])
        else:
            require(re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", spec) is not None, "invalidWindow")
            date = dt.date.fromisoformat(spec)
        following = date + dt.timedelta(days=1)
        first = dt.datetime.combine(date, dt.time(), zone).replace(fold=0)
        last = dt.datetime.combine(following, dt.time(), zone).replace(fold=0)
        start, end = first.astimezone(UTC), last.astimezone(UTC)
        require(start.astimezone(zone).date() == date and start < end, "skippedDate")
        require(end - start <= dt.timedelta(days=2), "windowLimit")
        return {"schemaVersion": 1, "kind": "calendarDayWindow", "status": "ready",
                "requestId": request["requestId"], "date": spec, "timeZone": zone_name,
                "observedAt": reference, "resolvedDate": date.isoformat(),
                "referenceOffsetSeconds": int(local_reference.utcoffset().total_seconds()),
                "startOffsetSeconds": int(first.utcoffset().total_seconds()),
                "endOffsetSeconds": int(last.utcoffset().total_seconds()),
                "window": {"start": start.isoformat(), "end": end.isoformat(), "timeZone": zone_name}}
    except ContextError as error:
        return {"schemaVersion": 1, "kind": "calendarDayWindow", "status": "unavailable", "errorCode": error.code}
    except (ValueError, OverflowError, TypeError):
        return {"schemaVersion": 1, "kind": "calendarDayWindow", "status": "unavailable", "errorCode": "invalidWindow"}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result)
        result[key] = value
    return result


def main() -> None:
    # Native callers additionally enforce wall time, pipe sizes and child cleanup.
    import resource
    resource.setrlimit(resource.RLIMIT_CPU, (3, 3))
    resource.setrlimit(resource.RLIMIT_AS, (384 * 1024 * 1024, 384 * 1024 * 1024))
    resource.setrlimit(resource.RLIMIT_FSIZE, (0, 0))
    sys.dont_write_bytecode = True

    def audit(event, arguments):
        if event.startswith("socket.") or event in {"subprocess.Popen", "os.system", "os.exec", "os.posix_spawn"}:
            raise PermissionError("calendar worker has no network or process authority")

    sys.addaudithook(audit)
    try:
        raw = sys.stdin.buffer.read(MAX_REQUEST_BYTES + 1)
        require(len(raw) <= MAX_REQUEST_BYTES, "requestLimit")
        request = json.loads(raw.decode("utf-8"), object_pairs_hook=unique_object,
                             parse_constant=lambda _: (_ for _ in ()).throw(ContextError("invalidRequest")))
        result = resolve_day(request) if type(request) is dict and request.get("kind") == "calendarDayWindow" else query(request)
    except ContextError as error:
        result = unavailable(error.code)
    except Exception:
        result = unavailable("invalidRequest")
    sys.stdout.buffer.write(encode(result) + b"\n")


if __name__ == "__main__":
    main()
