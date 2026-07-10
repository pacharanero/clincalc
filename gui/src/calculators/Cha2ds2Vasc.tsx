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
  SegmentedControl,
  Stack,
  Text,
  Textarea,
  ThemeIcon,
  Title,
  Tooltip,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconBrain,
  IconCheck,
  IconCopy,
  IconHeartbeat,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

type Sex = "male" | "female";

type Inputs = {
  age: number;
  sex: Sex;
  congestive_heart_failure: boolean;
  hypertension: boolean;
  diabetes: boolean;
  stroke_tia_thromboembolism: boolean;
  vascular_disease: boolean;
};

const blankInputs = (): Inputs => ({
  age: 70,
  sex: "male",
  congestive_heart_failure: false,
  hypertension: false,
  diabetes: false,
  stroke_tia_thromboembolism: false,
  vascular_disease: false,
});

const CRITERIA: Array<{
  key: keyof Pick<
    Inputs,
    | "congestive_heart_failure"
    | "hypertension"
    | "diabetes"
    | "stroke_tia_thromboembolism"
    | "vascular_disease"
  >;
  label: string;
  hint: string;
  points: string;
}> = [
  {
    key: "congestive_heart_failure",
    label: "Congestive heart failure / LV dysfunction",
    hint: "Signs/symptoms of heart failure, or moderate-to-severe LV systolic dysfunction.",
    points: "+1",
  },
  {
    key: "hypertension",
    label: "Hypertension",
    hint: "History of hypertension, treated or untreated.",
    points: "+1",
  },
  {
    key: "diabetes",
    label: "Diabetes mellitus",
    hint: "Type 1 or type 2 diabetes.",
    points: "+1",
  },
  {
    key: "stroke_tia_thromboembolism",
    label: "Prior stroke, TIA, or systemic arterial embolism",
    hint: "Two points. Do not use this for venous thromboembolism.",
    points: "+2",
  },
  {
    key: "vascular_disease",
    label: "Vascular disease",
    hint: "Prior MI, peripheral arterial disease, or aortic plaque. Excludes VTE.",
    points: "+1",
  },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const selected = CRITERIA.filter((c) => inputs[c.key])
    .map((c) => `- ${c.label} (${c.points})`)
    .join("\n");
  const agePoints = asString(working.age_points);
  const sexPoint = asString(working.sex_point);
  const recommendation = asString(working.recommendation);

  return [
    `CHA2DS2-VASc ${asString(r.result)}`,
    "",
    `Age: ${inputs.age} years (${agePoints || "0"} point${agePoints === "1" ? "" : "s"})`,
    `Sex: ${inputs.sex}${sexPoint ? ` (${sexPoint} point${sexPoint === "1" ? "" : "s"})` : ""}`,
    selected
      ? `Risk factors:\n${selected}`
      : "No non-age/sex criteria selected.",
    "",
    r.interpretation ?? "",
    "",
    `Recommendation band: ${recommendation || "not stated"}`,
    `Reference: ${r.reference ?? ""}`,
  ].join("\n");
}

function recommendationColour(recommendation: string, score: number): string {
  if (recommendation === "offer" || score >= 2) return "red";
  if (recommendation === "consider") return "yellow";
  return "teal";
}

export function Cha2ds2VascCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("cha2ds2vasc", inputs as unknown as Record<string, unknown>)
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

  const score = Number(response?.result ?? 0);
  const working = response?.working ?? {};
  const recommendation = asString(working.recommendation);
  const colour = useMemo(
    () => recommendationColour(recommendation, score),
    [recommendation, score],
  );

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "CHA2DS2-VASc result copied.",
        color: "teal",
        icon: <IconCheck size={18} />,
        autoClose: 2000,
      });
    } catch {
      notifications.show({
        title: "Copy failed",
        message: "Select the text and copy manually.",
        color: "red",
      });
    }
  };

  return (
    <Stack gap="xl" maw={960}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconBrain size={22} />
            </ThemeIcon>
            <Title order={2}>CHA2DS2-VASc</Title>
            <Badge color="teal" variant="light">
              Atrial fibrillation
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Stroke risk in non-valvular atrial fibrillation, guiding
            anticoagulation decisions. Female sex alone is treated as low risk
            in line with NICE NG196.
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
            <Group grow align="flex-end">
              <NumberInput
                label="Age"
                description="65-74 scores 1; 75+ scores 2"
                min={18}
                max={120}
                value={inputs.age}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, age: Number(value) || 0 }))
                }
              />
              <Box>
                <Text fw={500} size="sm" mb={4}>
                  Sex
                </Text>
                <SegmentedControl
                  fullWidth
                  value={inputs.sex}
                  onChange={(value) =>
                    setInputs((prev) => ({ ...prev, sex: value as Sex }))
                  }
                  data={[
                    { value: "male", label: "Male" },
                    { value: "female", label: "Female" },
                  ]}
                />
              </Box>
            </Group>

            <Divider />

            <Stack gap="sm">
              <Text fw={600}>Clinical criteria</Text>
              {CRITERIA.map((c) => (
                <Checkbox
                  key={c.key}
                  size="md"
                  label={
                    <Group gap="xs">
                      <Text>{c.label}</Text>
                      <Badge size="xs" variant="light" color="gray">
                        {c.points}
                      </Badge>
                    </Group>
                  }
                  description={c.hint}
                  checked={inputs[c.key]}
                  onChange={(e) => {
                    const checked = e.currentTarget.checked;
                    setInputs((prev) => ({ ...prev, [c.key]: checked }));
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
                  <Title order={1} c={colour} fz={64} lh={1}>
                    {String(response.result)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    / 9
                  </Text>
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    {recommendation || "—"}
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
                      Age: {asString(working.age_points) || "0"}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Sex: {asString(working.sex_point) || "0"}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Non-sex score:{" "}
                      {Math.max(0, score - Number(working.sex_point ?? 0))}
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
            <Group gap="xs" c="dimmed">
              <IconHeartbeat size={14} />
              <Text size="xs">Reference: {response.reference}</Text>
            </Group>
          </Stack>
        </Paper>
      )}
    </Stack>
  );
}
