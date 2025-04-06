#include "translation.h"
#include <string>
#include <map>
#include <future>
#include <iostream>
#include "common/definitions.h"
#include "translator/service.h"

using namespace marian::bergamot;

static bool isSupportedInternal(const TranslatorWrapper *translator, const std::string &from, const std::string &to);
static std::string translateInternal(TranslatorWrapper *translator, const std::string &from, const std::string &to, const std::string &input);

struct TranslatorWrapper
{
    std::map<std::string, marian::Ptr<TranslationModel>> models;
    AsyncService service;
    size_t numWorkers;

    explicit TranslatorWrapper(size_t workers)
        : service(AsyncService::Config{workers}), numWorkers(workers) {}

    ~TranslatorWrapper() = default;
};

extern "C" TranslatorWrapper *bergamot_create(size_t numWorkers)
{
    try
    {
        TranslatorWrapper *ptr = new TranslatorWrapper(numWorkers);
        return ptr;
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error creating translator: " << e.what() << std::endl;
        return nullptr;
    }
    catch (...)
    {
        std::cerr << "Unknown error creating translator" << std::endl;
        return nullptr;
    }
}

extern "C" void bergamot_destroy(TranslatorWrapper *translator)
{
    if (translator)
    {
        delete translator;
    }
}

extern "C" void bergamot_load_model_from_config(TranslatorWrapper *translator, const char *languagePair, const char *config)
{
    try
    {
        if (!translator || !languagePair || !config)
        {
            std::cerr << "Error: Invalid parameters for model loading" << std::endl;
            return;
        }

        std::string langPair = languagePair;
        auto options = parseOptionsFromString(config);
        MemoryBundle memoryBundle;
        translator->models[langPair] = marian::New<TranslationModel>(
            options, std::move(memoryBundle), translator->numWorkers);
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error loading model: " << e.what() << std::endl;
    }
    catch (...)
    {
        std::cerr << "Unknown error loading model" << std::endl;
    }
}

extern "C" bool bergamot_is_supported(TranslatorWrapper *translator, const char *from, const char *to)
{
    if (!translator || !from || !to)
    {
        return false;
    }

    try
    {
        return isSupportedInternal(translator, from, to) ||
               (isSupportedInternal(translator, from, "en") &&
                isSupportedInternal(translator, "en", to));
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error checking supported languages: " << e.what() << std::endl;
        return false;
    }
    catch (...)
    {
        std::cerr << "Unknown error checking supported languages" << std::endl;
        return false;
    }
}

extern "C" const char *bergamot_translate(TranslatorWrapper *translator, const char *from, const char *to, const char *input)
{
    if (!translator || !from || !to || !input)
    {
        std::cerr << "Error: Invalid parameters for translation" << std::endl;
        return nullptr;
    }

    char *c_result = nullptr;
    try
    {
        std::string result;
        if (isSupportedInternal(translator, from, to))
        {
            result = translateInternal(translator, from, to, input);
        }
        else if (isSupportedInternal(translator, from, "en") &&
                 isSupportedInternal(translator, "en", to))
        {
            std::string intermediateRes = translateInternal(translator, from, "en", input);
            if (intermediateRes.empty())
            {
                std::cerr << "Error: Intermediate translation produced empty result" << std::endl;
                return nullptr;
            }
            result = translateInternal(translator, "en", to, intermediateRes);
        }
        else
        {
            std::cerr << "Error: Unsupported language pair for translation: "
                      << from << " -> " << to << std::endl;
            return nullptr;
        }

        if (result.empty())
        {
            std::cerr << "Error: Translation produced empty result" << std::endl;
            return nullptr;
        }

        c_result = new char[result.size() + 1];
        std::copy(result.begin(), result.end(), c_result);
        c_result[result.size()] = '\0';

        return c_result;
    }
    catch (const std::exception &e)
    {
        delete[] c_result;
        std::cerr << "Error during translation: " << e.what() << std::endl;
        return nullptr;
    }
    catch (...)
    {
        delete[] c_result;
        std::cerr << "Unknown error during translation" << std::endl;
        return nullptr;
    }
}

extern "C" void bergamot_free_translation(const char *translation)
{
    delete[] translation;
}

static std::string translateInternal(TranslatorWrapper *translator,
                                     const std::string &from,
                                     const std::string &to,
                                     const std::string &input)
{
    std::string langPair = from + to;

    auto modelIt = translator->models.find(langPair);
    if (modelIt == translator->models.end())
    {
        std::cerr << "Error: Model not found for language pair: " << from << " -> " << to << std::endl;
        return "";
    }

    auto model = modelIt->second;
    ResponseOptions responseOptions;
    std::promise<Response> responsePromise;
    std::future<Response> responseFuture = responsePromise.get_future();

    auto callback = [&responsePromise](Response &&response)
    {
        responsePromise.set_value(std::move(response));
    };

    try
    {
        translator->service.translate(model, std::string(input), std::move(callback), responseOptions);
        if (!responseFuture.valid())
        {
            std::cerr << "Error: responseFuture is invalid!" << std::endl;
            return "";
        }
        auto status = responseFuture.wait_for(std::chrono::seconds(30));
        if (status != std::future_status::ready)
        {
            throw std::runtime_error("Translation timeout");
        }
        Response response = responseFuture.get();
        return response.target.text;
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error in translation service: " << e.what() << std::endl;
        return "";
    }
    catch (...)
    {
        std::cerr << "Unknown error in translation service" << std::endl;
        return "";
    }
}

static bool isSupportedInternal(const TranslatorWrapper *translator,
                                const std::string &from,
                                const std::string &to)
{
    if (!translator || from.empty() || to.empty())
    {
        return false;
    }

    std::string langPair = from + to;
    return translator->models.find(langPair) != translator->models.end();
}
