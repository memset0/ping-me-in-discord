#!/bin/sh
set -u

notify_error_channel=
notify_report_only=false
notify_print_session_id=false
notify_agent_name=${PINGME_AGENT_NAME-}
notify_project_name=${PINGME_PROJECT_NAME-}
notify_session_name=${PINGME_SESSION_NAME-}

notify_generic_session_id=${PINGME_SESSION_ID-}
notify_claude_session_id=${CLAUDE_CODE_SESSION_ID-}
notify_codex_thread_id=${CODEX_THREAD_ID-}
notify_session_id=$notify_generic_session_id
if [ -z "$notify_session_id" ]; then
    notify_session_id=$notify_claude_session_id
fi
if [ -z "$notify_session_id" ]; then
    notify_session_id=$notify_codex_thread_id
fi

while [ "$#" -gt 0 ]; do
    case "$1" in
        --agent-name)
            if [ "$#" -lt 2 ] || [ -z "$2" ]; then
                printf '%s\n' 'error: --agent-name requires a non-empty value' >&2
                exit 64
            fi
            notify_agent_name=$2
            shift 2
            ;;
        --project-name)
            if [ "$#" -lt 2 ] || [ -z "$2" ]; then
                printf '%s\n' 'error: --project-name requires a non-empty value' >&2
                exit 64
            fi
            notify_project_name=$2
            shift 2
            ;;
        --session-name)
            if [ "$#" -lt 2 ] || [ -z "$2" ]; then
                printf '%s\n' 'error: --session-name requires a non-empty value' >&2
                exit 64
            fi
            notify_session_name=$2
            shift 2
            ;;
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

if [ -z "$notify_agent_name" ]; then
    if [ -n "$notify_claude_session_id" ]; then
        notify_agent_name='Claude Code'
    elif [ -n "$notify_codex_thread_id" ]; then
        notify_agent_name=Codex
    else
        notify_agent_name=CLI
    fi
fi

if [ -z "$notify_project_name" ] && command -v git >/dev/null 2>&1; then
    notify_origin=$(git remote get-url origin 2>/dev/null || :)
    if [ -n "$notify_origin" ]; then
        notify_project_candidate=${notify_origin##*/}
        notify_project_candidate=${notify_project_candidate%.git}
        case "$notify_project_candidate" in
            ''|*:*|*@*) ;;
            *) notify_project_name=$notify_project_candidate ;;
        esac
    fi
    if [ -z "$notify_project_name" ]; then
        notify_project_root=$(git rev-parse --show-toplevel 2>/dev/null || :)
        notify_project_name=${notify_project_root##*/}
    fi
fi
if [ -z "$notify_project_name" ]; then
    notify_project_path=${PWD-}
    notify_project_name=${notify_project_path##*/}
fi
if [ -z "$notify_project_name" ]; then
    notify_project_name=unknown-project
fi

if [ -z "$notify_session_name" ]; then
    if [ -n "$notify_session_id" ]; then
        notify_session_remainder=${notify_session_id#????????}
        notify_session_prefix=${notify_session_id%"$notify_session_remainder"}
        notify_session_name=session-$notify_session_prefix
    else
        notify_session_name=interactive
    fi
fi

PINGME_AGENT_NAME=$notify_agent_name
PINGME_PROJECT_NAME=$notify_project_name
PINGME_SESSION_NAME=$notify_session_name
export PINGME_AGENT_NAME PINGME_PROJECT_NAME PINGME_SESSION_NAME
if [ -n "$notify_session_id" ]; then
    PINGME_SESSION_ID=$notify_session_id
    CODEX_THREAD_ID=$notify_session_id
    export PINGME_SESSION_ID CODEX_THREAD_ID
fi

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
