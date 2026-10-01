import { describe, expect, it, vi } from "vitest";

import { CommandError } from "./command-error";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { createProject, getProject } from "./commands";

describe("typed project commands", () => {
  it("returns the exact project shape from the command boundary", async () => {
    const project = {
      id: "project-id",
      name: "A project",
      description: "",
      status: "active",
      created_at: "2026-10-01T00:00:00Z",
      updated_at: "2026-10-01T00:00:00Z",
    };
    invokeMock.mockResolvedValueOnce(project);

    await expect(createProject({ name: "A project" })).resolves.toEqual(
      project,
    );
    expect(invokeMock).toHaveBeenCalledWith("project_create", {
      input: { name: "A project" },
    });
  });

  it("maps backend errors to safe user messages", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "Storage",
      details: { raw: "near SELECT: secret" },
    });

    const error = await getProject("missing").catch((value: unknown) => value);
    expect(error).toBeInstanceOf(CommandError);
    expect(error).toMatchObject({
      code: "storage",
      message: "The project could not be saved. Try again.",
    });
    expect((error as Error).message).not.toContain("secret");
  });
});
