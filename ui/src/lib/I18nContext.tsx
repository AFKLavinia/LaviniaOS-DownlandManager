import React, { createContext, useContext, useMemo } from "react";
import { translate, type I18nKey, type Lang } from "./i18n";

type TFn = (key: I18nKey, params?: Record<string, string | number>) => string;

const I18nContext = createContext<TFn>((key) => key);

export const I18nProvider: React.FC<{ lang: Lang; children: React.ReactNode }> = ({
  lang,
  children,
}) => {
  const t = useMemo<TFn>(() => (key, params) => translate(lang, key, params), [lang]);
  return <I18nContext.Provider value={t}>{children}</I18nContext.Provider>;
};

/** Hook returning the translate function for the active language. */
export const useT = (): TFn => useContext(I18nContext);
