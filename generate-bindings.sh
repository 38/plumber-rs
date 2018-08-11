#!/bin/bash
if [ -z "${BINDGEN}" ]
then
	BINDGEN=rust-bindgen
fi

if [ -z "$(/usr/bin/which ${BINDGEN})" ]
then
	echo "Cannot find binary rust-bindgen (hint: set environment variable BINDGEN)" >&2
	exit 1
fi

bindgen() {
	header=$1
	out=$(basename ${header} .h)_binding.rs
	echo "[BINDGEN] ${out}" >&2
	shift
	${BINDGEN} include/${header} -- ${CFLAGS} $@ > $(dirname $0)/generated/${out} || exit 1
}

bindgen plumber_api.h -I ${PLUMBER_PREFIX}/include/pservlet
bindgen va_list_helper.h
bindgen pstd.h -I ${PLUMBER_PREFIX}/include/pstd -I ${PLUMBER_PREFIX}/include/pservlet
