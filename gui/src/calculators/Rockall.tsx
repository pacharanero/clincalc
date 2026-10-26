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
  NumberInput,
  Paper,
  Select,
  SimpleGrid,
  Stack,
  Switch,
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
  IconDroplet,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * Rockall Score: pre-endoscopy (clinical) and full post-endoscopy risk of
 * rebleeding/mortality after acute upper GI haemorrhage (Rockall 1996).
 * Supplying `diagnosis` and `major_stigmata_of_recent_haemorrhage` together
 * computes the full score; the "Endoscopy performed" switch controls
 * whether those two fields are sent at all - clincalc rejects supplying
 * only one.
 */

type Comorbidity =
  | "nil_major"
  | "cardiac_failure_ischaemic_heart_disease_or_other_major"
  | "renal_failure_liver_failure_or_disseminated_malignancy";
type Diagnosis =
  | "mallory_weiss_or_no_lesion_identified"
  | "all_other_diagnoses"
  | "upper_gi_malignancy";

type Inputs = {
  age: number;
  systolic_bp_mm_hg: number;
  heart_rate_bpm: number;
  comorbidity: Comorbidity;
  endoscopyDone: boolean;
  diagnosis: Diagnosis;
  major_stigmata_of_recent_haemorrhage: boolean;
};

const blankInputs = (): Inputs => ({
  age: 65,
  systolic_bp_mm_hg: 120,
  heart_rate_bpm: 80,
  comorbidity: "nil_major",
  endoscopyDone: false,
  diagnosis: "all_other_diagnoses",
  major_stigmata_of_recent_haemorrhage: false,
});

const COMORBIDITY_OPTIONS: Array<{ value: Comorbidity; label: string }> = [
  { value: "nil_major", label: "Nil major (0)" },
  {
    value: "cardiac_failure_ischaemic_heart_disease_or_other_major",
    label: "Cardiac failure, IHD, or other major comorbidity (2)",
  },
  {
    value: "renal_failure_liver_failure_or_disseminated_malignancy",
    label: "Renal failure, liver failure, or disseminated malignancy (3)",
  },
];

const DIAGNOSIS_OPTIONS: Array<{ value: Diagnosis; label: string }> = [
  {
    value: "mallory_weiss_or_no_lesion_identified",
    label: "Mallory-Weiss tear, or no lesion identified (0)",
  },
  { value: "all_other_diagnoses", label: "All other diagnoses (1)" },
  { value: "upper_gi_malignancy", label: "Upper GI malignancy (2)" },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function label(options: Array<{ value: string; label: string }>, value: string): string {
  return options.find((o) => o.value === value)?.label ?? value;
}

function toPayload(inputs: Inputs): Record<string, unknown> {
  const { endoscopyDone, diagnosis, major_stigmata_of_recent_haemorrhage, ...rest } =
    inputs;
  if (!endoscopyDone) return rest;
  return {
    ...rest,
    diagnosis,
    major_stigmata_of_recent_haemorrhage,
  };
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const kind = asString(working.score_kind);
  const preEndoscopy = asString(working.pre_endoscopy_score);

  const lines = [
    `Rockall ${asString(r.result)} (${kind === "Full" ? "full, post-endoscopy" : "pre-endoscopy"})`,
    "",
    `Age: ${inputs.age} years`,
    `Shock: SBP ${inputs.systolic_bp_mm_hg} mmHg, HR ${inputs.heart_rate_bpm} bpm`,
    `Comorbidity: ${label(COMORBIDITY_OPTIONS, inputs.comorbidity)}`,
  ];
  if (inputs.endoscopyDone) {
    lines.push(`Diagnosis: ${label(DIAGNOSIS_OPTIONS, inputs.diagnosis)}`);
    lines.push(
      `Major stigmata of recent haemorrhage: ${inputs.major_stigmata_of_recent_haemorrhage ? "yes" : "no"}`,
    );
  } else {
    lines.push(`Pre-endoscopy subtotal: ${preEndoscopy}/7`);
  }
  lines.push("", r.interpretation ?? "");
  return lines.join("\n");
}

function scoreColour(kind: string, total: number): string {
  if (kind === "Full") {
    if (total >= 6) return "red";
    if (total >= 3) return "yellow";
    return "teal";
  }
  return total === 0 ? "teal" : "yellow";
}

export function RockallCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("rockall", toPayload(inputs))
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
  const kind = asString(working.score_kind);
  const total = Number(response?.result ?? 0);
  const colour = useMemo(() => scoreColour(kind, total), [kind, total]);
  const maxScore = kind === "Full" ? 11 : 7;

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "Rockall result copied.",
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
    <Stack gap="xl" maw={980}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconDroplet size={22} />
            </ThemeIcon>
            <Title order={2}>Rockall Score</Title>
            <Badge color="teal" variant="light">
              Upper GI bleeding
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Pre-endoscopy and full post-endoscopy risk of rebleeding and
            mortality after acute upper GI haemorrhage (Rockall 1996).
            Complements the pre-endoscopy-only Glasgow-Blatchford score.
          </Text>
        </Box>
        <Tooltip label="Reset to defaults">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="md">
            <SimpleGrid cols={3} spacing="md">
              <NumberInput
                label="Age"
                description="years"
                min={0}
                max={130}
                value={inputs.age}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, age: Number(value) || 0 }))
                }
              />
              <NumberInput
                label="Systolic BP"
                description="mmHg"
                min={1}
                max={300}
                value={inputs.systolic_bp_mm_hg}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    systolic_bp_mm_hg: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Heart rate"
                description="bpm"
                min={1}
                max={250}
                value={inputs.heart_rate_bpm}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    heart_rate_bpm: Number(value) || 0,
                  }))
                }
              />
            </SimpleGrid>

            <Select
              label="Comorbidity"
              data={COMORBIDITY_OPTIONS}
              value={inputs.comorbidity}
              onChange={(value) =>
                value &&
                setInputs((prev) => ({
                  ...prev,
                  comorbidity: value as Comorbidity,
                }))
              }
              allowDeselect={false}
            />

            <Divider />

            <Switch
              label="Endoscopy performed"
              description="Add the diagnosis and stigmata to compute the full post-endoscopy score"
              checked={inputs.endoscopyDone}
              onChange={(e) => {
                const checked = e.currentTarget.checked;
                setInputs((prev) => ({ ...prev, endoscopyDone: checked }));
              }}
            />

            {inputs.endoscopyDone && (
              <>
                <Select
                  label="Endoscopic diagnosis"
                  data={DIAGNOSIS_OPTIONS}
                  value={inputs.diagnosis}
                  onChange={(value) =>
                    value &&
                    setInputs((prev) => ({
                      ...prev,
                      diagnosis: value as Diagnosis,
                    }))
                  }
                  allowDeselect={false}
                />
                <Checkbox
                  size="md"
                  label="Major stigmata of recent haemorrhage"
                  description="Blood in the upper GI tract, an adherent clot, or a visible/spurting vessel"
                  checked={inputs.major_stigmata_of_recent_haemorrhage}
                  onChange={(e) => {
                    const checked = e.currentTarget.checked;
                    setInputs((prev) => ({
                      ...prev,
                      major_stigmata_of_recent_haemorrhage: checked,
                    }));
                  }}
                />
              </>
            )}
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
                  <Title order={1} c={colour} fz={64} lh={1}>
                    {String(response.result)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    / {maxScore}
                  </Text>
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    {kind === "Full" ? "full" : "pre-endoscopy"}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                {kind === "Full" && (
                  <>
                    <Divider my={4} />
                    <Badge variant="outline" color="gray" w="fit-content">
                      Pre-endoscopy subtotal: {asString(working.pre_endoscopy_score)}/7
                    </Badge>
                  </>
                )}
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
              minRows={9}
              maxRows={12}
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
