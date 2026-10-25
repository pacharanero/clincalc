// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";
import {
  ActionIcon,
  Alert,
  Badge,
  Box,
  Button,
  Card,
  Divider,
  Group,
  Loader,
  Paper,
  Select,
  Stack,
  Text,
  Textarea,
  ThemeIcon,
  Title,
  Tooltip,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconAlertCircle,
  IconCheck,
  IconCopy,
  IconEye,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * GCS: three-component bedside conscious-level score (Teasdale & Jennett
 * 1974). Each component carries its own "not testable" (NT) option - the
 * official guidance is that a total/band must never be reported when any
 * component is untestable, so the form always offers NT and the result
 * card reflects whichever total/band `clincalc` actually returns (it may
 * be withheld).
 */

type EyeOpening =
  | "spontaneous"
  | "to_sound"
  | "to_pressure"
  | "no_response"
  | "not_testable";
type VerbalResponse =
  | "orientated"
  | "confused"
  | "words"
  | "sounds"
  | "no_response"
  | "not_testable";
type MotorResponse =
  | "obey_commands"
  | "localising"
  | "normal_flexion"
  | "abnormal_flexion"
  | "extension"
  | "no_response"
  | "not_testable";

type Inputs = {
  eye_opening: EyeOpening;
  verbal_response: VerbalResponse;
  motor_response: MotorResponse;
};

const blankInputs = (): Inputs => ({
  eye_opening: "spontaneous",
  verbal_response: "orientated",
  motor_response: "obey_commands",
});

const EYE_OPTIONS: Array<{ value: EyeOpening; label: string }> = [
  { value: "spontaneous", label: "Spontaneous (4)" },
  { value: "to_sound", label: "To sound (3)" },
  { value: "to_pressure", label: "To pressure (2)" },
  { value: "no_response", label: "No eye opening (1)" },
  { value: "not_testable", label: "Not testable, e.g. swollen shut (NT)" },
];

const VERBAL_OPTIONS: Array<{ value: VerbalResponse; label: string }> = [
  { value: "orientated", label: "Orientated (5)" },
  { value: "confused", label: "Confused conversation (4)" },
  { value: "words", label: "Words, not a conversation (3)" },
  { value: "sounds", label: "Sounds only (2)" },
  { value: "no_response", label: "No verbal response (1)" },
  { value: "not_testable", label: "Not testable, e.g. intubated (NT)" },
];

const MOTOR_OPTIONS: Array<{ value: MotorResponse; label: string }> = [
  { value: "obey_commands", label: "Obeys commands (6)" },
  { value: "localising", label: "Localises to pressure (5)" },
  { value: "normal_flexion", label: "Normal flexion / withdrawal (4)" },
  { value: "abnormal_flexion", label: "Abnormal (decorticate) flexion (3)" },
  { value: "extension", label: "Extension / decerebrate posturing (2)" },
  { value: "no_response", label: "No motor response (1)" },
  { value: "not_testable", label: "Not testable, e.g. paralysed (NT)" },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function label(options: Array<{ value: string; label: string }>, value: string): string {
  return options.find((o) => o.value === value)?.label ?? value;
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const total = r.result === "not_testable" ? "NT" : asString(r.result);
  const level = asString(working.level);

  return [
    `GCS ${total}${level ? ` (${level})` : ""}`,
    "",
    `Eyes (E): ${label(EYE_OPTIONS, inputs.eye_opening)}`,
    `Verbal (V): ${label(VERBAL_OPTIONS, inputs.verbal_response)}`,
    `Motor (M): ${label(MOTOR_OPTIONS, inputs.motor_response)}`,
    "",
    r.interpretation ?? "",
    "",
    `Reference: ${r.reference ?? ""}`,
  ].join("\n");
}

export function GcsCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("gcs", inputs as unknown as Record<string, unknown>)
      .then((r) => {
        if (cancelled) return;
        setResponse(r);
        setClipboardText(buildClipboardSummary(r, inputs));
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setError(String(e));
        setResponse(null);
      })
      .finally(() => !cancelled && setPending(false));
    return () => {
      cancelled = true;
    };
  }, [inputs]);

  const working = response?.working ?? {};
  const notTestable = response?.result === "not_testable";
  const level = asString(working.level);

  const scoreColour = useMemo(() => {
    if (!level) return "gray";
    if (level === "severe") return "red";
    if (level === "moderate") return "yellow";
    return "teal";
  }, [level]);

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "GCS result copied.",
        color: "teal",
        icon: <IconCheck size={18} />,
        autoClose: 2000,
      });
    } catch {
      notifications.show({
        title: "Copy failed",
        message: "Select the text above and copy manually.",
        color: "red",
        icon: <IconAlertCircle size={18} />,
      });
    }
  };

  return (
    <Stack gap="xl" maw={920}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconEye size={22} />
            </ThemeIcon>
            <Title order={2}>Glasgow Coma Scale</Title>
            <Badge color="teal" variant="light">
              Conscious level
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Bedside score of conscious level from eye, verbal, and motor
            response (Teasdale &amp; Jennett 1974). A total and band are only
            reported when all three components are testable.
          </Text>
        </Box>
        <Tooltip label="Reset to normal">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="md">
            <Select
              label="Eye opening (E)"
              data={EYE_OPTIONS}
              value={inputs.eye_opening}
              onChange={(value) =>
                value &&
                setInputs((prev) => ({
                  ...prev,
                  eye_opening: value as EyeOpening,
                }))
              }
              allowDeselect={false}
            />
            <Select
              label="Verbal response (V)"
              data={VERBAL_OPTIONS}
              value={inputs.verbal_response}
              onChange={(value) =>
                value &&
                setInputs((prev) => ({
                  ...prev,
                  verbal_response: value as VerbalResponse,
                }))
              }
              allowDeselect={false}
            />
            <Select
              label="Motor response (M)"
              data={MOTOR_OPTIONS}
              value={inputs.motor_response}
              onChange={(value) =>
                value &&
                setInputs((prev) => ({
                  ...prev,
                  motor_response: value as MotorResponse,
                }))
              }
              allowDeselect={false}
            />
          </Stack>
        </Card>

        <Card
          withBorder
          padding="lg"
          radius="lg"
          style={{ flex: 1 }}
          bg="var(--mantine-color-default-hover)"
        >
          <Stack gap="md" h="100%">
            <Text fw={600} c="dimmed" size="sm" tt="uppercase">
              Result
            </Text>
            {pending && !response && (
              <Group gap="xs">
                <Loader size="xs" />
                <Text c="dimmed">Computing…</Text>
              </Group>
            )}
            {error && (
              <Text c="red" size="sm">
                {error}
              </Text>
            )}
            {response && !notTestable && (
              <>
                <Group align="baseline" gap="xs">
                  <Title order={1} c={scoreColour} fz={64} lh={1}>
                    {String(response.result)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    / 15
                  </Text>
                  <Badge ml="auto" color={scoreColour} variant="light" size="lg">
                    {level || "—"}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Group gap="xs" wrap="wrap">
                  <Badge variant="outline" color="gray">
                    E: {asString(working.eye_opening_score)}
                  </Badge>
                  <Badge variant="outline" color="gray">
                    V: {asString(working.verbal_response_score)}
                  </Badge>
                  <Badge variant="outline" color="gray">
                    M: {asString(working.motor_response_score)}
                  </Badge>
                </Group>
              </>
            )}
            {response && notTestable && (
              <Alert
                color="orange"
                icon={<IconAlertCircle size={18} />}
                title="Total withheld - component(s) not testable"
              >
                {response.interpretation}
              </Alert>
            )}
          </Stack>
        </Card>
      </Group>

      {response && (
        <Paper
          withBorder
          radius="lg"
          p="lg"
          className="clipboard-preview"
          style={{ borderColor: "var(--mantine-color-teal-4)", borderWidth: 2 }}
        >
          <Stack gap="sm">
            <Group justify="space-between" align="center">
              <Box>
                <Text fw={700} size="md">
                  Paste-ready summary
                </Text>
                <Text size="xs" c="dimmed">
                  Edit freely before copying - your edits are preserved.
                </Text>
              </Box>
              <Button
                leftSection={<IconCopy size={16} />}
                color="teal"
                onClick={copy}
              >
                Copy result
              </Button>
            </Group>
            <Textarea
              autosize
              minRows={8}
              maxRows={8}
              value={clipboardText}
              onChange={(e) => setClipboardText(e.currentTarget.value)}
              styles={{ input: { background: "var(--mantine-color-body)" } }}
            />
            <Text size="xs" c="dimmed">
              Reference: {response.reference}
            </Text>
          </Stack>
        </Paper>
      )}
    </Stack>
  );
}
