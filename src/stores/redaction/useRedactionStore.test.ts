import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { renderPage } = vi.hoisted(() => ({ renderPage: vi.fn() }));

vi.mock("../../application/redactionClient", () => ({
  redactionClient: { renderPage },
}));

import { useRedactionStore } from "./useRedactionStore";

function renderedPage(page: number) {
  return {
    page,
    aspectRatio: 0.75,
    imageDataUrl: `data:image/jpeg;base64,page-${page}`,
    words: [],
  };
}

function deferred<T>() {
  let resolve: (value: T) => void;
  const promise = new Promise<T>((complete) => {
    resolve = complete;
  });
  return { promise, resolve: resolve! };
}

describe("redaction page navigation", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    renderPage.mockReset();
  });

  function storeWithSource(pageCount = 5) {
    const store = useRedactionStore();
    store.source = { path: "/documents/source.pdf", name: "source.pdf", pageCount };
    return store;
  }

  it("updates the requested page before its preview has rendered", async () => {
    const preview = deferred<ReturnType<typeof renderedPage>>();
    renderPage.mockReturnValue(preview.promise);
    const store = storeWithSource();

    const navigation = store.goToPage(2);

    expect(store.currentPage).toBe(2);
    expect(store.renderedPage).toBeNull();
    expect(store.loadingPage).toBe(true);

    preview.resolve(renderedPage(2));
    await navigation;

    expect(store.renderedPage?.page).toBe(2);
    expect(store.loadingPage).toBe(false);
  });

  it("preloads only valid neighboring pages", async () => {
    renderPage.mockImplementation((_sourcePath: string, page: number) =>
      Promise.resolve(renderedPage(page)),
    );
    const store = storeWithSource(3);

    await store.goToPage(1);

    expect(renderPage.mock.calls.map(([, page]) => page)).toEqual([1, 2]);
  });

  it("deduplicates concurrent renders for the same page", async () => {
    const preview = deferred<ReturnType<typeof renderedPage>>();
    renderPage.mockReturnValue(preview.promise);
    const store = storeWithSource();

    const firstNavigation = store.goToPage(2);
    const secondNavigation = store.goToPage(2);

    expect(renderPage).toHaveBeenCalledTimes(1);

    preview.resolve(renderedPage(2));
    await Promise.all([firstNavigation, secondNavigation]);
  });

  it("keeps the latest requested page visible when earlier renders finish later", async () => {
    const secondPage = deferred<ReturnType<typeof renderedPage>>();
    const thirdPage = deferred<ReturnType<typeof renderedPage>>();
    renderPage.mockImplementation((_sourcePath: string, page: number) => {
      if (page === 2) return secondPage.promise;
      if (page === 3) return thirdPage.promise;
      return Promise.resolve(renderedPage(page));
    });
    const store = storeWithSource();

    const secondNavigation = store.goToPage(2);
    const thirdNavigation = store.goToPage(3);
    secondPage.resolve(renderedPage(2));
    await secondNavigation;

    expect(store.currentPage).toBe(3);
    expect(store.renderedPage).toBeNull();

    thirdPage.resolve(renderedPage(3));
    await thirdNavigation;

    expect(store.renderedPage?.page).toBe(3);
  });

  it("evicts previews outside the active page neighborhood", async () => {
    renderPage.mockImplementation((_sourcePath: string, page: number) =>
      Promise.resolve(renderedPage(page)),
    );
    const store = storeWithSource(5);

    await store.goToPage(2);
    await store.goToPage(4);
    await store.goToPage(2);

    expect(renderPage.mock.calls.filter(([, page]) => page === 2)).toHaveLength(2);
  });
});
