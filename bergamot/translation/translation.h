#ifndef TRANSLATION_H
#define TRANSLATION_H

#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct TranslatorWrapper TranslatorWrapper;

    TranslatorWrapper *bergamot_create(size_t numWorkers);
    void bergamot_destroy(TranslatorWrapper *translator);
    void bergamot_load_model_from_config(TranslatorWrapper *translator, const char *languagePair, const char *config);
    bool bergamot_is_supported(TranslatorWrapper *translator, const char *from, const char *to);
    const char *bergamot_translate(TranslatorWrapper *translator, const char *from, const char *to, const char *input);
    void bergamot_free_translation(const char *translation);

#ifdef __cplusplus
}
#endif

#endif // TRANSLATION_H
