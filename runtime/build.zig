const std = @import("std");

pub fn build(b: *std.build.Builder) void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});

    // Standard release options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall.
    const mode = b.standardReleaseOptions();

    const exe = b.addExecutable("runtime", "src/main.zig");
    exe.setTarget(target);
    exe.setBuildMode(mode);
    exe.install();

    exe.addCSourceFiles(&.{
        // actor
        "ponyc/src/libponyrt/actor/actor.c",
        "ponyc/src/libponyrt/actor/messageq.c",
        // asio
        "ponyc/src/libponyrt/asio/asio.c",
        "ponyc/src/libponyrt/asio/emscripten.c",
        "ponyc/src/libponyrt/asio/epoll.c",
        "ponyc/src/libponyrt/asio/event.c",
        "ponyc/src/libponyrt/asio/iocp.c",
        "ponyc/src/libponyrt/asio/kqueue.c",
        // ds
        "ponyc/src/libponyrt/ds/fun.c",
        "ponyc/src/libponyrt/ds/hash.c",
        "ponyc/src/libponyrt/ds/list.c",
        "ponyc/src/libponyrt/ds/stack.c",
        // gc
        "ponyc/src/libponyrt/gc/actormap.c",
        "ponyc/src/libponyrt/gc/cycle.c",
        "ponyc/src/libponyrt/gc/delta.c",
        "ponyc/src/libponyrt/gc/gc.c",
        "ponyc/src/libponyrt/gc/objectmap.c",
        "ponyc/src/libponyrt/gc/serialise.c",
        "ponyc/src/libponyrt/gc/trace.c",
        // lang
        "ponyc/src/libponyrt/lang/directory.c",
        "ponyc/src/libponyrt/lang/errno.c",
        "ponyc/src/libponyrt/lang/io.c",
        "ponyc/src/libponyrt/lang/lsda.c",
        "ponyc/src/libponyrt/lang/paths.c",
        "ponyc/src/libponyrt/lang/posix_except.c",
        "ponyc/src/libponyrt/lang/process.c",
        "ponyc/src/libponyrt/lang/socket.c",
        "ponyc/src/libponyrt/lang/ssl.c",
        "ponyc/src/libponyrt/lang/stat.c",
        "ponyc/src/libponyrt/lang/stdfd.c",
        "ponyc/src/libponyrt/lang/time.c",
        "ponyc/src/libponyrt/lang/win_except.c",
        // mem
        "ponyc/src/libponyrt/mem/alloc.c",
        "ponyc/src/libponyrt/mem/heap.c",
        "ponyc/src/libponyrt/mem/pagemap.c",
        "ponyc/src/libponyrt/mem/pool.c",
        // options
        "ponyc/src/libponyrt/options/options.c",
        // platform
        "ponyc/src/libponyrt/platform/ponyassert.c",
        "ponyc/src/libponyrt/platform/threads.c",
        // sched
        "ponyc/src/libponyrt/sched/cpu.c",
        "ponyc/src/libponyrt/sched/mpmcq.c",
        "ponyc/src/libponyrt/sched/mutemap.c",
        "ponyc/src/libponyrt/sched/scheduler.c",
        "ponyc/src/libponyrt/sched/start.c",
        "ponyc/src/libponyrt/sched/systematic_testing.c",
    }, &.{
        "-Wall",
        "-W",
        "-Wstrict-prototypes",
        "-Wwrite-strings",
        "-Wno-missing-field-initializers",
        "-DPONY_VERSION_STR=\"zig-built-rt\"",
    });

    exe.addIncludePath("ponyc/src/common/");
    exe.addIncludePath("ponyc/src/libponyrt/");
    // TODO macOS only
    exe.addIncludePath("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include");

    const run_cmd = exe.run();
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const exe_tests = b.addTest("src/main.zig");
    exe_tests.setTarget(target);
    exe_tests.setBuildMode(mode);

    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&exe_tests.step);
}
