import localforage from 'localforage';

export const kvStorage = localforage.createInstance({
  driver: [
    localforage.WEBSQL,
    localforage.INDEXEDDB,
    localforage.LOCALSTORAGE
  ],
  name: 'wl-admin-kv-storage',
});
