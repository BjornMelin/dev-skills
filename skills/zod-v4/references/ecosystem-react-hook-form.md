# Ecosystem: React Hook Form (RHF)

## Basic Integration

```ts
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

const Schema = z.object({
  name: z.string().min(1, { error: "Required" }),
  email: z.email({ error: "Invalid email" }),
  age: z.coerce.number().pipe(z.number().min(18, { error: "Must be 18+" })),
});

type FormData = z.infer<typeof Schema>;

export function MyForm() {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<FormData>({ resolver: zodResolver(Schema) });

  return (
    <form onSubmit={handleSubmit((data) => console.log(data))}>
      <input {...register("name")} />
      {errors.name?.message}
    </form>
  );
}
```

## Notes

- Prefer `z.coerce.*().pipe(z.*())` for form/query values so validation runs on coerced outputs.
- For server-side validation of the same schema, use `safeParse` and `z.flattenError`.
