function prompt_char {
    git branch >/dev/null 2>/dev/null && echo '$' && return
    hg root >/dev/null 2>/dev/null && echo '$' && return
    echo '$'
}

function virtualenv_info {
    [[ -n "$VIRTUAL_ENV" ]] && echo '('${VIRTUAL_ENV:t}') '
}

export FRS_TERM_PID=$$
function frs_info {
    frs prompt
}

CRUNCH_BRACKET_COLOR="%{$fg[white]%}"
CRUNCH_TIME_COLOR="%{$fg[yellow]%}"
kamiya_at="%{$fg[white]%}.%{$reset_color%}"
kamiya_time_info=" $CRUNCH_BRACKET_COLOR"'['"$CRUNCH_TIME_COLOR%*$CRUNCH_BRACKET_COLOR"']'"%{$reset_color%}"

PROMPT='%F{038}%n%f'${kamiya_at}'%F{011}%m%f'${kamiya_time_info}' in %B%F{012}%~%f%b$(git_prompt_info)$(ruby_prompt_info)
%F{038}λ%f$(frs_info)
%F{011}$(virtualenv_info)%F{white}$(prompt_char) '

ZSH_THEME_GIT_PROMPT_PREFIX=' on %F{magenta}'
ZSH_THEME_GIT_PROMPT_SUFFIX='%f'
ZSH_THEME_GIT_PROMPT_DIRTY='%F{green}!'
ZSH_THEME_GIT_PROMPT_UNTRACKED='%F{green}?'
ZSH_THEME_GIT_PROMPT_CLEAN=''

ZSH_THEME_RUBY_PROMPT_PREFIX=' using %F{red}'
ZSH_THEME_RUBY_PROMPT_SUFFIX='%f'
