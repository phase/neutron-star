const std = @import("std");
const pony = @cImport({
    @cInclude("pony.h");
});

// in libponyrt/sched/start.c
extern "c" fn pony_init(argc: c_int, argv: [*c][*c]u8) c_int;

pub fn main() !void {
    // setup arena allocator
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    // load command line arguments to send to pony_init
    const argv: [][:0]u8 = try std.process.argsAlloc(allocator);
    const argc = @intCast(c_int, argv.len);
    const argv_ptr = @ptrCast([*c][*c]u8, argv.ptr);

    std.debug.print("Calling pony_init with {d} args.\n", .{argc});
    // see https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyc/codegen/genexe.c#L118
    _ = pony_init(argc, argv_ptr);
    const ctx: ?*pony.pony_ctx_t = pony.pony_ctx();
    _ = ctx;
    std.debug.print("Finished!\n", .{});
}
