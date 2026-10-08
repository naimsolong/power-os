import { useMutation } from "@tanstack/react-query";
import {
  type AiChatRequest,
  type AiChatResponse,
  sendAiMessage,
} from "@/lib/api";

export function useSendAiMessage() {
  return useMutation<AiChatResponse, Error, AiChatRequest>({
    mutationFn: sendAiMessage,
  });
}
