#!/bin/sh
set -u

notify_error_channel=
notify_report_only=false
notify_print_session_id=false

notify_session_id=${CLAUDE_CODE_SESSION_ID-}
if [ -z "$notify_session_id" ]; then
    notify_session_id=${CODEX_THREAD_ID-}
fi
if [ -n "$notify_session_id" ]; then
    CODEX_THREAD_ID=$notify_session_id
    export CODEX_THREAD_ID
fi

while [ "$#" -gt 0 ]; do
    case "$1" in
        --error-channel)
            if [ "$#" -lt 2 ]; then
                printf '%s\n' 'error: --error-channel requires a value' >&2
                exit 64
            fi
            notify_error_channel=$2
            shift 2
            ;;
        --report-only)
            notify_report_only=true
            shift
            ;;
        --print-session-id)
            notify_print_session_id=true
            shift
            ;;
        --)
            shift
            break
            ;;
        *) break ;;
    esac
done

notify_report_failure() {
    if [ -n "$notify_error_channel" ]; then
        pingme report-error --channel "$notify_error_channel"
    else
        pingme report-error
    fi
}

if [ "$notify_print_session_id" = true ]; then
    if [ "$notify_report_only" = true ] || [ "$#" -ne 0 ]; then
        printf '%s\n' 'error: --print-session-id cannot be combined with another mode or pingme arguments' >&2
        exit 64
    fi
    if [ -z "$notify_session_id" ]; then
        printf '%s\n' 'error: no coding-agent session ID is available' >&2
        exit 65
    fi
    printf '%s\n' "$notify_session_id"
    exit 0
fi

if [ "$notify_report_only" = true ]; then
    if [ "$#" -ne 0 ]; then
        printf '%s\n' 'error: --report-only does not accept pingme arguments' >&2
        exit 64
    fi
    notify_report_failure
    exit $?
fi

if [ "$#" -eq 0 ]; then
    printf '%s\n' 'error: provide pingme arguments after --' >&2
    exit 64
fi

if [ "$1" = report-error ]; then
    printf '%s\n' 'error: report-error must not be wrapped recursively' >&2
    exit 64
fi

pingme "$@"
notify_status=$?
if [ "$notify_status" -eq 0 ]; then
    exit 0
fi

if ! notify_report_failure; then
    printf '%s\n' 'warning: the Discord failure report also failed; not retrying' >&2
fi
exit "$notify_status"
