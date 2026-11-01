-- Organisations d'un utilisateur, lisibles AVANT le choix d'une organisation active (F01 §3 : liste de choix après la connexion).
-- La RLS forcée cache `organizations` tant qu'aucune organisation n'est active : cette fonction SECURITY DEFINER rend, pour l'utilisateur
-- du contexte (app.user_id), ses propres adhésions actives et les champs d'affichage de leurs organisations, rien d'autre.
-- En avant seulement ; DATABASE_SCHEMA.md §6.4 (own_memberships).

SET lock_timeout = '5s';

CREATE FUNCTION my_organizations()
    RETURNS TABLE (org_id uuid, org_name text, plan text, kind text, lifecycle text, role text)
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
    SELECT o.id, o.name, o.plan, o.kind, o.lifecycle, m.role
      FROM memberships m
      JOIN organizations o ON o.id = m.org_id
     WHERE m.user_id = current_actor_id()
       AND m.status = 'active'
       AND o.lifecycle <> 'closed'
     ORDER BY o.name, o.id
$$;
REVOKE ALL ON FUNCTION my_organizations() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION my_organizations() TO immo_app;

-- Le propriétaire (qui exécute la fonction) lit memberships sous FORCE RLS : il faut la politique « ses propres adhésions »
-- côté propriétaire, limitée à l'utilisateur du contexte et sans organisation active (même condition que pour immo_app).
CREATE POLICY own_memberships_owner ON memberships FOR SELECT TO immo_owner
    USING (user_id = current_actor_id() AND current_org_id() IS NULL);
