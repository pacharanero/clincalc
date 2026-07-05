// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";
import {
  ActionIcon,
  Badge,
  Box,
  Button,
  Card,
  Checkbox,
  Divider,
  Group,
  Loader,
  Paper,
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
  IconRefresh,
  IconThermometer,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * FeverPAIN: five-item validated score guiding antibiotic prescribing in
 * acute sore throat (Little et al, Lancet Infect Dis 2014).
 *
 * The five inputs are independent booleans, so the form is trivial - the
 * design effort goes into:
 *   1. Instant recompute on every change (no Calculate button to forget).
 *   2. A prominent, editable clipboard-preview textarea (the "soft
 *      interoperability" headline). The text is pre-built but the user
 *      can tweak before copying, so clinician edits survive.
 *   3. A working/breakdown panel that explains the score, so a clinician
 *      can sense-check it without leaving the calculator.
 */

const CRITERIA: Array<{
  key: string;
  label: string;
  hint: string;
}> = [
  {
    key: "fever",
    label: "Fever in the last 24 hours",
    hint: "Reported or measured fever within the past day.",
  },
  {
    key: "purulence",
    label: "Purulence on the tonsils",
    hint: "Visible pus on the tonsillar surface.",
  },
  {
    key: "attend_rapidly",
    label: "Attended within 3 days of symptom onset",
    hint: "Rapid attendance suggests more severe disease.",
  },
  {
    key: "inflamed_tonsils",
    label: "Severely inflamed tonsils",
    hint: "Markedly red, swollen tonsils on examination.",
  },
  {
    key: "absence_of_cough",
    label: "No cough or coryza",
    hint: "Absence of cough/coryza tilts toward bacterial cause.",
  },
];

type Inputs = Record<string, boolean>;

const blankInputs = (): Inputs =>
  Object.fromEntries(CRITERIA.map((c) => [c.key, false]));

/**
 * The default clipboard summary. FeverPAIN's `working` map already
 * contains the score, level, prescribing recommendation, and streptococcus
 * isolation band; we lay them out in a way a GP would actually paste.
 *
 * Defensive: read every `working` key through a helper that gracefully
 * handles missing values, so a future clincalc schema change can never
 * crash this UI - it just renders empty fields.
 */
function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const score = r.result;
  const rec = asString(working.prescribing_recommendation);
  const strep = asString(working.streptococcus_rate);
  const ticked = CRITERIA.filter((c) => inputs[c.key])
    .map((c) => `- ${c.label}`)
    .join("\n");

  return [
    `FeverPAIN ${asString(score)} / 5`,
    "",
    r.interpretation ?? "",
    "",
    ticked.length > 0 ? "Positive criteria:\n" + ticked : "No criteria met.",
    "",
    `Prescribing: ${rec || "(not stated)"}`,
    `Streptococcus isolation: ${strep || "(not stated)"}`,
    "",
    `Reference: ${r.reference ?? ""}`,
  ].join("\n");
}

export function FeverPainCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  // Recompute on every change. FeverPAIN is so cheap to score that there
  // is no value in debouncing - the round trip to Rust is sub-millisecond.
  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("feverpain", inputs as Record<string, unknown>)
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

  const score = response?.result ?? 0;
  const working = response?.working ?? {};
  const level = asString(working.level);
  const strepRate = asString(working.streptococcus_rate);
  const prescribing = asString(working.prescribing_recommendation);

  // Colour-code the score tile by the FeverPAIN bands (Little 2014):
  //   0-1  no antibiotic, low strep yield   -> green
  //   2-3  delayed prescribing              -> amber
  //   4-5  immediate antibiotic considered  -> red
  const scoreColor = useMemo(() => {
    const n = typeof score === "number" ? score : Number(score);
    if (n >= 4) return "red";
    if (n >= 2) return "yellow";
    return "teal";
  }, [score]);

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "Result on the clipboard - paste anywhere.",
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
              <IconThermometer size={22} />
            </ThemeIcon>
            <Title order={2}>FeverPAIN</Title>
            <Badge color="teal" variant="light">
              Acute sore throat
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Five-item score guiding antibiotic prescribing in acute sore throat
            (Little et al, Lancet Infect Dis 2014). Validated in adults and
            children aged 3+.
          </Text>
        </Box>
        <Tooltip label="Reset all criteria">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="md">
            <Text fw={600}>Tick each criterion that applies</Text>
            <Divider />
            <Stack gap="sm">
              {CRITERIA.map((c) => (
                <Checkbox
                  key={c.key}
                  size="md"
                  label={c.label}
                  description={c.hint}
                  checked={inputs[c.key]}
                  onChange={(e) => {
                    // Read .checked SYNCHRONOUSLY here, before the
                    // functional setState updater closes over `e`. React
                    // 19's concurrent renderer can defer the updater
                    // until after the synthetic event has finished
                    // propagating, at which point `currentTarget` has
                    // been nulled out and reading `.checked` throws
                    // "null is not an object". Capture the boolean now,
                    // refer to it (not `e`) inside the updater.
                    const checked = e.currentTarget.checked;
                    setInputs((prev) => ({
                      ...prev,
                      [c.key]: checked,
                    }));
                  }}
                />
              ))}
            </Stack>
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

            {response && (
              <>
                <Group align="baseline" gap="xs">
                  <Title order={1} c={scoreColor} fz={64} lh={1}>
                    {String(score)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    / 5
                  </Text>
                  <Badge ml="auto" color={scoreColor} variant="light" size="lg">
                    {level || "—"}
                  </Badge>
                </Group>

                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>

                <Divider my={4} />

                <Stack gap={4}>
                  <Text size="xs" c="dimmed" tt="uppercase" fw={600}>
                    Working
                  </Text>
                  <Group gap="xs" wrap="wrap">
                    <Badge variant="outline" color="gray">
                      Strep isolation: {strepRate || "—"}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Prescribing: {prescribing || "—"}
                    </Badge>
                  </Group>
                </Stack>
              </>
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
          style={{
            // Bring the headline-feature card visually forward.
            borderColor: "var(--mantine-color-teal-4)",
            borderWidth: 2,
          }}
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
              minRows={9}
              value={clipboardText}
              onChange={(e) => setClipboardText(e.currentTarget.value)}
              styles={{
                input: {
                  background: "var(--mantine-color-body)",
                },
              }}
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
