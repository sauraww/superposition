{-# LANGUAGE ForeignFunctionInterface #-}
{-# OPTIONS_GHC -Wno-incomplete-patterns #-}
{-# OPTIONS_GHC -Wno-unrecognised-pragmas #-}
{-# HLINT ignore "Use camelCase" #-}

module Client
( expStartPolling
, getExpClient
, createExpClient
, getApplicableVariants
, getSatisfiedExperiments
, getRunningExperiments
) where

import           Data.Aeson.Types
import           Data.Functor          (($>))
import           Foreign               (FunPtr, Ptr)
import           Foreign.C             (CInt (..), CShort (..), CULong (..))
import           Foreign.C.String
import           Foreign.ForeignPtr
import           Foreign.Marshal.Alloc (free)
import           Foreign.Ptr           (nullPtr)
import           Prelude

data Arc_Client

type ExpClient = Arc_Client

type CTenant = CString
type Tenant = String

type Error = String

foreign import ccall unsafe "new_client"
    c_new_exp_client :: CTenant -> CULong -> CString -> IO CInt

foreign import ccall unsafe "&free_client"
    c_free_exp_client :: FunPtr (Ptr ExpClient -> IO ())

foreign import ccall unsafe "get_client"
    c_get_exp_client :: CTenant -> IO (Ptr ExpClient)

foreign import ccall unsafe "last_error_message"
    c_last_error_message :: IO CString

foreign import ccall unsafe "&free_string"
    c_free_string :: FunPtr (CString -> IO ())

foreign import ccall unsafe "start_polling_update"
    c_start_polling_update :: CTenant -> IO ()

foreign import ccall unsafe "get_applicable_variant"
    c_get_applicable_variants :: Ptr ExpClient -> CString -> CShort -> IO CString

foreign import ccall unsafe "get_satisfied_experiments"
    c_get_satisfied_experiments :: Ptr ExpClient -> CString -> IO CString

foreign import ccall unsafe "get_running_experiments"
    c_get_running_experiments :: Ptr ExpClient -> IO CString

expStartPolling :: Tenant -> IO ()
expStartPolling tenant =
    newCAString tenant
    >>= newForeignPtr c_free_string
    >>= flip withForeignPtr c_start_polling_update

getError :: IO String
getError = c_last_error_message
            >>= newForeignPtr c_free_string
            >>= flip withForeignPtr peekCAString

cleanup :: [Ptr a] -> IO ()
cleanup items = mapM free items $> ()

createExpClient:: Tenant -> Integer -> String -> IO (Either Error ())
createExpClient tenant frequency hostname = do
    let duration = fromInteger frequency
    cTenant   <- newCAString tenant
    cHostname <- newCAString hostname
    resp      <- c_new_exp_client cTenant duration cHostname
    _         <- cleanup [cTenant, cHostname]
    case resp of
        0 -> pure $ Right ()
        _ -> Left <$> getError

getExpClient :: Tenant -> IO (Either Error (ForeignPtr ExpClient))
getExpClient tenant = do
    cTenant   <- newCAString tenant
    cacClient <- c_get_exp_client cTenant
    _         <- cleanup [cTenant]
    if cacClient == nullPtr
        then Left <$> getError
        else Right <$> newForeignPtr c_free_exp_client cacClient

getApplicableVariants :: ForeignPtr ExpClient -> String -> Integer -> IO (Either Error String)
getApplicableVariants client query toss = do
    context  <- newCAString query
    variants <- withForeignPtr client (\c -> c_get_applicable_variants c context (fromInteger toss))
    _        <- cleanup [context]
    if variants == nullPtr
        then Left <$> getError
        else do
            fptrVariants  <- newForeignPtr c_free_string variants
            Right <$> withForeignPtr fptrVariants peekCAString
            -- pure $
                -- case fromJSON variantVector of
                    -- Error s     -> Left s
                    -- Success vec -> Right vec

getSatisfiedExperiments :: ForeignPtr ExpClient -> String -> IO (Either Error Value)
getSatisfiedExperiments client query = do
    context     <- newCAString query
    experiments <- withForeignPtr client (`c_get_satisfied_experiments` context)
    _           <- cleanup [context]
    if experiments == nullPtr
        then Left <$> getError
        else do
            fptrExperiments  <- newForeignPtr c_free_string experiments
            Right . toJSON <$> withForeignPtr fptrExperiments peekCAString

getRunningExperiments :: ForeignPtr ExpClient -> IO (Either Error Value)
getRunningExperiments client = do
    experiments <- withForeignPtr client c_get_running_experiments
    if experiments == nullPtr
        then Left <$> getError
        else do
            fptrExperiments  <- newForeignPtr c_free_string experiments
            Right . toJSON <$> withForeignPtr fptrExperiments peekCAString
