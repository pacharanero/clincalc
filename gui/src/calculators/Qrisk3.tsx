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
  Checkbox,
  Divider,
  Group,
  Loader,
  NumberInput,
  Paper,
  Select,
  SimpleGrid,
  Stack,
  Text,
  Textarea,
  ThemeIcon,
  Title,
  Tooltip,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconAlertTriangle,
  IconCheck,
  IconCopy,
  IconHeartRateMonitor,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

type Sex = "male" | "female";
type Ethnicity =
  | "white_or_not_stated"
  | "indian"
  | "pakistani"
  | "bangladeshi"
  | "other_asian"
  | "black_caribbean"
  | "black_african"
  | "chinese"
  | "other_ethnic_group";
type Smoking =
  | "non_smoker"
  | "ex_smoker"
  | "light_smoker"
  | "moderate_smoker"
  | "heavy_smoker";

type Inputs = {
  age: number;
  sex: Sex;
  ethnicity: Ethnicity;
  smoking: Smoking;
  height_cm: number;
  weight_kg: number;
  cholesterol_hdl_ratio: number;
  systolic_bp: number;
  systolic_bp_sd: number;
  townsend: number;
  atrial_fibrillation: boolean;
  atypical_antipsychotic: boolean;
  regular_steroids: boolean;
  migraine: boolean;
  rheumatoid_arthritis: boolean;
  ckd_stage_3_5: boolean;
  severe_mental_illness: boolean;
  sle: boolean;
  treated_hypertension: boolean;
  type1_diabetes: boolean;
  type2_diabetes: boolean;
  erectile_dysfunction: boolean;
  family_history_chd: boolean;
};

const blankInputs = (): Inputs => ({
  age: 64,
  sex: "female",
  ethnicity: "white_or_not_stated",
  smoking: "non_smoker",
  height_cm: 170,
  weight_kg: 75,
  cholesterol_hdl_ratio: 4,
  systolic_bp: 140,
  systolic_bp_sd: 0,
  townsend: 0,
  atrial_fibrillation: false,
  atypical_antipsychotic: false,
  regular_steroids: false,
  migraine: false,
  rheumatoid_arthritis: false,
  ckd_stage_3_5: false,
  severe_mental_illness: false,
  sle: false,
  treated_hypertension: false,
  type1_diabetes: false,
  type2_diabetes: false,
  erectile_dysfunction: false,
  family_history_chd: false,
});

const ETHNICITY_OPTIONS: Array<{ value: Ethnicity; label: string }> = [
  { value: "white_or_not_stated", label: "White or not stated" },
  { value: "indian", label: "Indian" },
  { value: "pakistani", label: "Pakistani" },
  { value: "bangladeshi", label: "Bangladeshi" },
  { value: "other_asian", label: "Other Asian" },
  { value: "black_caribbean", label: "Black Caribbean" },
  { value: "black_african", label: "Black African" },
  { value: "chinese", label: "Chinese" },
  { value: "other_ethnic_group", label: "Other ethnic group" },
];

const SMOKING_OPTIONS: Array<{ value: Smoking; label: string }> = [
  { value: "non_smoker", label: "Non-smoker" },
  { value: "ex_smoker", label: "Ex-smoker" },
  { value: "light_smoker", label: "Light smoker (<10/day)" },
  { value: "moderate_smoker", label: "Moderate smoker (10-19/day)" },
  { value: "heavy_smoker", label: "Heavy smoker (20+/day)" },
];

const CONDITIONS: Array<{
  key: keyof Pick<
    Inputs,
    | "atrial_fibrillation"
    | "atypical_antipsychotic"
    | "regular_steroids"
    | "migraine"
    | "rheumatoid_arthritis"
    | "ckd_stage_3_5"
    | "severe_mental_illness"
    | "sle"
    | "treated_hypertension"
    | "type1_diabetes"
    | "type2_diabetes"
    | "erectile_dysfunction"
    | "family_history_chd"
  >;
  label: string;
  hint: string;
}> = [
  {
    key: "atrial_fibrillation",
    label: "Atrial fibrillation",
    hint: "Current or previous AF.",
  },
  {
    key: "treated_hypertension",
    label: "Treated hypertension",
    hint: "Diagnosis of hypertension and currently on antihypertensive treatment.",
  },
  {
    key: "family_history_chd",
    label: "Family history of CHD <60",
    hint: "First-degree relative with angina or heart attack before 60.",
  },
  {
    key: "type1_diabetes",
    label: "Type 1 diabetes",
    hint: "Do not also tick type 2 diabetes.",
  },
  {
    key: "type2_diabetes",
    label: "Type 2 diabetes",
    hint: "Do not also tick type 1 diabetes.",
  },
  {
    key: "ckd_stage_3_5",
    label: "CKD stage 3-5",
    hint: "CKD stages 1-2 do not count for QRISK3.",
  },
  {
    key: "rheumatoid_arthritis",
    label: "Rheumatoid arthritis",
    hint: "Clinician-asserted diagnosis.",
  },
  {
    key: "sle",
    label: "Systemic lupus erythematosus",
    hint: "Clinician-asserted diagnosis.",
  },
  {
    key: "migraine",
    label: "Migraine",
    hint: "Current or previous migraine diagnosis.",
  },
  {
    key: "severe_mental_illness",
    label: "Severe mental illness",
    hint: "Schizophrenia, bipolar disorder, or severe depression.",
  },
  {
    key: "atypical_antipsychotic",
    label: "Atypical antipsychotic",
    hint: "Currently prescribed an atypical antipsychotic.",
  },
  {
    key: "regular_steroids",
    label: "Regular oral corticosteroids",
    hint: "Regular systemic steroid treatment.",
  },
  {
    key: "erectile_dysfunction",
    label: "Erectile dysfunction",
    hint: "Male equation only; ignored for females.",
  },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function oneDecimal(v: unknown): string {
  const n = typeof v === "number" ? v : Number(v);
  if (!Number.isFinite(n)) return "";
  return n.toFixed(1);
}

function optionLabel<T extends string>(
  options: Array<{ value: T; label: string }>,
  value: T,
): string {
  return options.find((o) => o.value === value)?.label ?? value;
}

function riskColour(risk: number): string {
  if (risk >= 20) return "red";
  if (risk >= 10) return "yellow";
  return "teal";
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const selected = CONDITIONS.filter((c) => inputs[c.key])
    .map((c) => `- ${c.label}`)
    .join("\n");
  const working = r.working ?? {};

  return [
    `QRISK3 10-year CVD risk: ${oneDecimal(r.result)}%`,
    "",
    r.interpretation ?? "",
    "",
    `Age/sex: ${inputs.age} years, ${inputs.sex}`,
    `Ethnicity: ${optionLabel(ETHNICITY_OPTIONS, inputs.ethnicity)}`,
    `Smoking: ${optionLabel(SMOKING_OPTIONS, inputs.smoking)}`,
    `BMI used: ${oneDecimal(working.bmi)} kg/m2 (derived from ${inputs.height_cm} cm / ${inputs.weight_kg} kg)`,
    `Cholesterol/HDL ratio: ${inputs.cholesterol_hdl_ratio}`,
    `Systolic BP: ${inputs.systolic_bp} mmHg; SBP SD: ${inputs.systolic_bp_sd} mmHg`,
    `Townsend score: ${inputs.townsend}`,
    selected
      ? `Risk factors:\n${selected}`
      : "No listed QRISK3 clinical risk factors selected.",
    "",
    `Reference: ${r.reference ?? ""}`,
    "",
    asString(working.disclaimer),
  ].join("\n");
}

export function Qrisk3Calculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("qrisk3", inputs as unknown as Record<string, unknown>)
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

  const risk = Number(response?.result ?? 0);
  const colour = useMemo(() => riskColour(risk), [risk]);
  const working = response?.working ?? {};
  const disclaimer = asString(working.disclaimer);

  const reset = () => setInputs(blankInputs());

  const setNumber = (key: keyof Inputs) => (value: string | number) => {
    setInputs((prev) => ({ ...prev, [key]: Number(value) || 0 }));
  };

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "QRISK3 result copied.",
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
    <Stack gap="xl" maw={1120}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconHeartRateMonitor size={22} />
            </ThemeIcon>
            <Title order={2}>QRISK3</Title>
            <Badge color="teal" variant="light">
              10-year CVD risk
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            UK primary-care cardiovascular risk model. BMI is derived from
            height and weight; smoking and ethnicity use the exact QRISK3
            categories.
          </Text>
        </Box>
        <Tooltip label="Reset to defaults">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1.35 }}>
          <Stack gap="md">
            <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="md">
              <NumberInput
                label="Age"
                description="Validated for 25-84"
                min={25}
                max={84}
                value={inputs.age}
                onChange={setNumber("age")}
              />
              <Select
                label="Sex"
                value={inputs.sex}
                data={[
                  { value: "female", label: "Female" },
                  { value: "male", label: "Male" },
                ]}
                allowDeselect={false}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    sex: (value ?? "female") as Sex,
                    erectile_dysfunction:
                      value === "male" ? prev.erectile_dysfunction : false,
                  }))
                }
              />
              <Select
                label="Ethnicity"
                value={inputs.ethnicity}
                data={ETHNICITY_OPTIONS}
                allowDeselect={false}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    ethnicity: (value ?? "white_or_not_stated") as Ethnicity,
                  }))
                }
              />
              <Select
                label="Smoking"
                value={inputs.smoking}
                data={SMOKING_OPTIONS}
                allowDeselect={false}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    smoking: (value ?? "non_smoker") as Smoking,
                  }))
                }
              />
              <NumberInput
                label="Height"
                description="cm"
                min={1}
                decimalScale={1}
                value={inputs.height_cm}
                onChange={setNumber("height_cm")}
              />
              <NumberInput
                label="Weight"
                description="kg"
                min={1}
                decimalScale={1}
                value={inputs.weight_kg}
                onChange={setNumber("weight_kg")}
              />
              <NumberInput
                label="Cholesterol / HDL ratio"
                description="Use the ratio, not separate lipids"
                min={0}
                step={0.1}
                decimalScale={1}
                value={inputs.cholesterol_hdl_ratio}
                onChange={setNumber("cholesterol_hdl_ratio")}
              />
              <NumberInput
                label="Systolic BP"
                description="mmHg"
                min={0}
                value={inputs.systolic_bp}
                onChange={setNumber("systolic_bp")}
              />
              <NumberInput
                label="Systolic BP SD"
                description="0 if unknown - not the BP itself"
                min={0}
                value={inputs.systolic_bp_sd}
                onChange={setNumber("systolic_bp_sd")}
              />
              <NumberInput
                label="Townsend score"
                description="0 if unknown"
                step={0.1}
                decimalScale={1}
                value={inputs.townsend}
                onChange={setNumber("townsend")}
              />
            </SimpleGrid>

            <Divider />

            <Stack gap="sm">
              <Text fw={600}>Clinical risk factors</Text>
              <SimpleGrid cols={{ base: 1, md: 2 }} spacing="sm">
                {CONDITIONS.map((c) => {
                  const disabled =
                    c.key === "erectile_dysfunction" && inputs.sex !== "male";
                  return (
                    <Checkbox
                      key={c.key}
                      size="md"
                      label={c.label}
                      description={disabled ? "Male equation only" : c.hint}
                      disabled={disabled}
                      checked={inputs[c.key]}
                      onChange={(e) => {
                        const checked = e.currentTarget.checked;
                        setInputs((prev) => ({ ...prev, [c.key]: checked }));
                      }}
                    />
                  );
                })}
              </SimpleGrid>
            </Stack>
          </Stack>
        </Card>

        <Card
          withBorder
          padding="lg"
          radius="lg"
          style={{ flex: 0.85 }}
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
                    {oneDecimal(response.result)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    %
                  </Text>
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    {risk >= 10 ? "NICE threshold met" : "Below 10%"}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Group gap="xs" wrap="wrap">
                  <Badge variant="outline" color="gray">
                    BMI: {oneDecimal(working.bmi)} kg/m2
                  </Badge>
                  <Badge variant="outline" color="gray">
                    Equation: {asString(working.sex) || inputs.sex}
                  </Badge>
                </Group>
                {disclaimer && (
                  <Alert
                    mt="auto"
                    color="yellow"
                    variant="light"
                    icon={<IconAlertTriangle size={18} />}
                    title="QRISK3 licence notice"
                  >
                    <Text size="xs" style={{ lineHeight: 1.45 }}>
                      {disclaimer}
                    </Text>
                  </Alert>
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
              minRows={10}
              maxRows={10}
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
