#define _GNU_SOURCE
#include <dlfcn.h>
#include <netdb.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <stdint.h>
#include <netinet/in.h>
#include <systemd/sd-bus.h>

void mindrove_scope_shim_anchor(void) {}

void *dlopen(const char *path, int flags) {
    static void *(*real_dlopen)(const char *, int);

    if (!real_dlopen)
        real_dlopen = dlsym(RTLD_NEXT, "dlopen");

    if (path && strstr(path, "libMindRoveWrapper.so")) {
        fprintf(stderr, "MindRove dlopen flags: %#x\n", flags);
        flags &= ~RTLD_DEEPBIND;
    }

    return real_dlopen(path, flags);
}

static unsigned resolve_scope(const struct in6_addr *target) {
    sd_bus *bus = NULL;
    sd_bus_message *reply = NULL;
    sd_bus_error error = SD_BUS_ERROR_NULL;
    unsigned scope = 0;

    if (sd_bus_open_system(&bus) < 0)
        goto out;

    if (sd_bus_call_method(
        bus, "org.freedesktop.resolve1",
        "/org/freedesktop/resolve1",
        "org.freedesktop.resolve1.Manager",
        "ResolveHostname",
        &error, &reply, "isit",
        0, "syncbox.local", AF_INET6, UINT64_C(0)
    ) < 0)
        goto out;

    if (sd_bus_message_enter_container(reply, 'a', "(iiay)") <= 0)
        goto out;

    while (sd_bus_message_enter_container(reply, 'r', "iiay") > 0) {
        int index, family;
        const void *address;
        size_t length;

        if (sd_bus_message_read(reply, "ii", &index, &family) < 0 ||
            sd_bus_message_read_array(reply, 'y', &address, &length) < 0)
            break;

        if (index > 0 && family == AF_INET6 &&
            length == sizeof(*target) &&
            memcmp(target, address, length) == 0)
            scope = (unsigned)index;

        if (sd_bus_message_exit_container(reply) < 0 || scope)
            break;
    }

out:
    sd_bus_error_free(&error);
    sd_bus_message_unref(reply);
    sd_bus_flush_close_unref(bus);
    return scope;
}

int getaddrinfo(
    const char *node,
    const char *service,
    const struct addrinfo *hints,
    struct addrinfo **res
) {
    static int (*original)(
        const char *, const char *,
        const struct addrinfo *, struct addrinfo **
    );

    if (!original)
        original = dlsym(RTLD_NEXT, "getaddrinfo");

    int rc = original(node, service, hints, res);
    if (rc != 0 || !node || !res ||
        strcmp(node, "syncbox.local") != 0)
        return rc;

    for (struct addrinfo *ai = *res; ai; ai = ai->ai_next) {
        if (ai->ai_family != AF_INET6 ||
            !ai->ai_addr ||
            ai->ai_addrlen < sizeof(struct sockaddr_in6))
            continue;

        struct sockaddr_in6 *sa =
            (struct sockaddr_in6 *)ai->ai_addr;

        if (IN6_IS_ADDR_LINKLOCAL(&sa->sin6_addr) &&
            sa->sin6_scope_id == 0) {
            sa->sin6_scope_id = resolve_scope(&sa->sin6_addr);

            fprintf(stderr, "Resolved IPv6 scope: %u\n",
                    sa->sin6_scope_id);
        }
    }

    return rc;
}
